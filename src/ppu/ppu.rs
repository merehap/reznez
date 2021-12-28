use crate::ppu::address::Address;
use crate::ppu::clock::Clock;
use crate::ppu::memory;
use crate::ppu::memory::Memory;
use crate::ppu::name_table::background_tile_index::BackgroundTileIndex;
use crate::ppu::name_table::name_table::NameTable;
use crate::ppu::name_table::name_table_number::NameTableNumber;
use crate::ppu::oam::Oam;
use crate::ppu::palette::palette_table::PaletteTable;
use crate::ppu::pattern_table::PatternTable;
use crate::ppu::register::ctrl::{Ctrl, VBlankNmi};
use crate::ppu::register::mask::Mask;
use crate::ppu::register::status::Status;
use crate::ppu::frame::Frame;

const FIRST_VBLANK_CYCLE: u64 = 3 * 27384;
const SECOND_VBLANK_CYCLE: u64 = 3 * 57165;

pub struct Ppu {
    memory: Memory,
    oam: Oam,
    status: Status,

    clock: Clock,

    is_nmi_period: bool,
    vram_address: Address,
    next_vram_upper_byte: Option<u8>,
    vram_data: u8,
}

impl Ppu {
    pub fn new(memory: Memory) -> Ppu {
        Ppu {
            memory,
            oam: Oam::new(),
            status: Status::new(),

            clock: Clock::new(),

            // TODO: Is this the same as vblank_active?
            is_nmi_period: false,
            vram_address: Address::from_u16(0),
            next_vram_upper_byte: None,
            vram_data: 0,
        }
    }

    pub fn status(&self) -> Status {
        self.status
    }

    pub fn clock(&self) -> &Clock {
        &self.clock
    }

    #[inline]
    pub fn pattern_table(&self) -> PatternTable {
        self.memory.pattern_table()
    }

    #[inline]
    pub fn name_table(&self, number: NameTableNumber) -> NameTable {
        self.memory.name_table(number)
    }

    pub fn palette_table(&self) -> PaletteTable {
        self.memory.palette_table()
    }

    pub fn write_oam(&mut self, oam_address: u8, value: u8) {
        self.oam[oam_address] = value;
    }

    pub fn update_vram_data(&mut self, ctrl: Ctrl) {
        self.vram_data = self.memory[self.vram_address];
        let increment = ctrl.vram_address_increment as u8;
        self.vram_address = self.vram_address.advance(increment);
    }

    pub fn write_vram(&mut self, ctrl: Ctrl, value: u8) {
        self.memory[self.vram_address] = value;
        let increment = ctrl.vram_address_increment as u8;
        self.vram_address = self.vram_address.advance(increment);
    }

    pub fn write_partial_vram_address(&mut self, value: u8) {
        if let Some(upper) = self.next_vram_upper_byte {
            self.vram_address = Address::from_u16(((upper as u16) << 8) + value as u16);
            self.next_vram_upper_byte = None;
        } else {
            self.next_vram_upper_byte = Some(value);
        }
    }

    pub fn nmi_enabled(&self, ctrl: Ctrl) -> bool {
        self.is_nmi_period && ctrl.vblank_nmi == VBlankNmi::On
    }

    pub fn step(&mut self, ctrl: Ctrl, mask: Mask, frame: &mut Frame) -> StepResult {
        let frame_started = self.clock().is_first_cycle_of_frame();
        if frame_started {
            println!(
                "PPU Cycle: {}, Frame: {}",
                self.clock().total_cycles(),
                self.clock().frame(),
            );
        }

        let total_cycles = self.clock().total_cycles();
        let mut step_result = StepResult {
            status: self.status,
            vram_data: self.vram_data,
            nmi_trigger: false,
        };

        // When reading palette data, read the current data pointed to
        // by self.vram_address, not what was previously pointed to.
        if self.vram_address >= memory::PALETTE_TABLE_START {
            step_result.vram_data = self.memory[self.vram_address];
        }

        // TODO: Fix the first and second vblank cycles to not be special-cased if possible.
        if total_cycles == FIRST_VBLANK_CYCLE || total_cycles == SECOND_VBLANK_CYCLE {
            step_result.nmi_trigger = true;
        } else if total_cycles < SECOND_VBLANK_CYCLE {
            // Do nothing.
        } else if self.clock.scanline() == 241 && self.clock.cycle() == 1 {
            self.is_nmi_period = true;
            self.status.vblank_active = true;
            step_result.nmi_trigger = true;
        } else if self.clock.scanline() == 261 && self.clock.cycle() == 1 {
            self.is_nmi_period = false;
            self.status.vblank_active = false;
        } else if self.clock.scanline() == 1 && self.clock.cycle() == 1 {
            if mask.background_enabled {
                self.render_background(ctrl, frame);
            }

            if mask.sprites_enabled {
                self.render_sprites(ctrl, frame);
            }
        }

        self.clock.tick(self.rendering_enabled(mask));
        step_result
    }

    fn render_background(&mut self, ctrl: Ctrl, frame: &mut Frame) {
        let palette_table = self.memory.palette_table();
        frame.set_universal_background_rgb(palette_table.universal_background_rgb());

        let name_table_number = ctrl.name_table_number;
        let background_table_side = ctrl.background_table_side;
        for background_tile_index in BackgroundTileIndex::iter() {
            let (pattern_index, palette_table_index) =
                self.name_table(name_table_number).tile_entry_at(background_tile_index);
            let pixel_column = 8 * background_tile_index.column();
            let start_row = 8 * background_tile_index.row();
            for row_in_tile in 0..8 {
                self.pattern_table().render_tile_sliver(
                    background_table_side,
                    pattern_index,
                    row_in_tile as usize,
                    false,
                    palette_table.background_palette(palette_table_index),
                    frame.background_tile_sliver(pixel_column, start_row + row_in_tile),
                    );
            }
        }
    }

    fn render_sprites(&mut self, ctrl: Ctrl, frame: &mut Frame) {
        frame.clear_sprite_buffer();

        let palette_table = self.memory.palette_table();
        let sprite_table_side = ctrl.sprite_table_side;
        for sprite in self.oam.sprites() {
            let x = sprite.x_coordinate();
            let y = sprite.y_coordinate();
            let palette_table_index = sprite.palette_table_index();

            for row_in_tile in 0..8 {
                if y + row_in_tile >= 240 {
                    break;
                }

                let y =
                    if sprite.flip_vertically() {
                        y + 7 - row_in_tile
                    } else {
                        y + row_in_tile
                    };

                let mut sliver = frame.sprites_tile_sliver(x, y);
                sliver.1 = sprite.priority();

                self.pattern_table().render_tile_sliver(
                    sprite_table_side,
                    sprite.pattern_index(),
                    row_in_tile as usize,
                    sprite.flip_horizontally(),
                    palette_table.sprite_palette(palette_table_index),
                    &mut sliver.0,
                    );
            }
        }
    }

    pub fn stop_vblank(&mut self) {
        self.status.vblank_active = false;
    }

    fn rendering_enabled(&self, mask: Mask) -> bool {
        mask.sprites_enabled || mask.background_enabled
    }
}

#[derive(Clone, Copy)]
pub struct StepResult {
    status: Status,
    vram_data: u8,
    nmi_trigger: bool,
}

impl StepResult {
    pub fn normal(status: Status, vram_data: u8) -> StepResult {
        StepResult {
            status,
            nmi_trigger: false,
            vram_data,
        }
    }

    pub fn trigger_nmi(status: Status, vram_data: u8) -> StepResult {
        StepResult {
            status,
            nmi_trigger: true,
            vram_data,
        }
    }

    pub fn status(&self) -> Status {
        self.status
    }

    pub fn vram_data(&self) -> u8 {
        self.vram_data
    }

    pub fn nmi_trigger(&self) -> bool {
        self.nmi_trigger
    }
}
