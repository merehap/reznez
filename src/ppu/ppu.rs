use crate::ppu::address::Address;
use crate::ppu::clock::Clock;
use crate::ppu::memory::Memory;
use crate::ppu::name_table::NameTable;
use crate::ppu::name_table_number::NameTableNumber;
use crate::ppu::oam::Oam;
use crate::ppu::pattern_table::PatternTable;
use crate::ppu::register::ctrl::{Ctrl, VBlankNmi};
use crate::ppu::register::mask::Mask;
use crate::ppu::screen::Screen;
use crate::ppu::tile_number::TileNumber;

const FIRST_VBLANK_CYCLE: u64 = 3 * 27384;
const SECOND_VBLANK_CYCLE: u64 = 3 * 57165;

pub struct Ppu {
    memory: Memory,
    oam: Oam,
    ctrl: Ctrl,
    mask: Mask,

    clock: Clock,

    is_nmi_period: bool,
    vram_address: Address,
    next_vram_upper_byte: Option<u8>,
    oam_address: u8,
}

impl Ppu {
    pub fn new(memory: Memory) -> Ppu {
        Ppu {
            memory,
            oam: Oam::new(),
            ctrl: Ctrl::new(),
            mask: Mask::new(),

            clock: Clock::new(),

            is_nmi_period: false,
            vram_address: Address::from_u16(0),
            next_vram_upper_byte: None,
            oam_address: 0,
        }
    }

    pub fn clock(&self) -> &Clock {
        &self.clock
    }

    pub fn ctrl(&self) -> Ctrl {
        self.ctrl
    }

    pub fn set_ctrl(&mut self, ctrl: Ctrl) {
        println!("Setting PPUCTRL: {:?}", ctrl);
        self.ctrl = ctrl;
    }

    pub fn set_mask(&mut self, mask: Mask) {
        println!("Setting PPUMASK: {:?}", mask);
        self.mask = mask;
    }

    pub fn oam_address(&self) -> u8 {
        self.oam_address
    }

    pub fn set_oam_address(&mut self, value: u8) {
        self.oam_address = value;
    }

    pub fn write_oam(&mut self, value: u8) {
        self.oam[self.oam_address] = value;
        // TODO: Verify that wrapping is the correct behavior.
        self.oam_address = self.oam_address.wrapping_add(1);
    }

    pub fn write_vram(&mut self, value: u8) {
        println!("Writing to VRAM Address: [{}]={}", self.vram_address, value);
        self.memory[self.vram_address] = value;
        let increment = self.ctrl.vram_address_increment() as u8;
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

    pub fn nmi_enabled(&self) -> bool {
        self.is_nmi_period && self.ctrl.vblank_nmi() == VBlankNmi::On
    }

    pub fn step(&mut self, screen: &mut Screen) -> StepEvents {
        let frame_started = self.clock().is_start_of_frame();
        if frame_started {
            println!(
                "PPU Cycle: {}, Frame: {}",
                self.clock().total_cycles(),
                self.clock().frame(),
                );
        }

        let total_cycles = self.clock().total_cycles();
        let step_event;
        // TODO: Fix the first and second vblank cycles to not be special-cased if possible.
        if total_cycles == FIRST_VBLANK_CYCLE || total_cycles == SECOND_VBLANK_CYCLE {
            step_event = StepEvents::start_vblank()
        } else if total_cycles < SECOND_VBLANK_CYCLE {
            step_event = StepEvents::no_events()
        } else if self.clock.scanline() == 241 && self.clock.cycle() == 1 {
            self.is_nmi_period = true;
            step_event = StepEvents::start_vblank();
        } else if self.clock.scanline() == 261 && self.clock.cycle() == 1 {
            self.is_nmi_period = false;
            step_event = StepEvents::stop_vblank();
        } else if self.clock.cycle() == 1 {
            self.render_background(screen);
            self.render_sprites(screen);
            step_event = StepEvents::no_events();
        } else {
            step_event = StepEvents::no_events();
        }

        self.clock.tick(self.rendering_enabled());
        step_event
    }

    fn render_background(&mut self, screen: &mut Screen) {
        let palette_table = self.memory.palette_table();
        screen.set_universal_background_rgb(palette_table.universal_background_rgb());

        let name_table_number = self.ctrl.name_table_number();
        let background_table_side = self.ctrl.background_table_side();
        for tile_number in TileNumber::iter() {
            let (tile_index, palette_table_index) =
                self.name_table(name_table_number).tile_entry_at(tile_number);
            let pixel_column = 8 * tile_number.column();
            let start_row = 8 * tile_number.row();
            for row_in_tile in 0..8 {
                self.pattern_table().render_tile_sliver(
                    background_table_side,
                    tile_index,
                    row_in_tile as usize,
                    palette_table.background_palette(palette_table_index),
                    screen.background_tile_sliver(pixel_column, start_row + row_in_tile),
                    );
            }
        }
    }

    fn render_sprites(&mut self, screen: &mut Screen) {
        screen.clear_sprite_buffer();

        let palette_table = self.memory.palette_table();
        let sprite_table_side = self.ctrl.sprite_table_side();
        for sprite in self.oam.sprites() {
            let x = sprite.x_coordinate();
            let y = sprite.y_coordinate();
            let tile_index = 32 * x + y;
            let palette_table_index = sprite.palette_table_index();

            for row_in_tile in 0..8 {
                if y + row_in_tile >= 240 {
                    break;
                }

                let mut sliver = screen.sprites_tile_sliver(x, y + row_in_tile);
                sliver.1 = sprite.priority();

                self.pattern_table().render_tile_sliver(
                    sprite_table_side,
                    tile_index,
                    row_in_tile as usize,
                    palette_table.sprite_palette(palette_table_index),
                    &mut sliver.0,
                    );
            }
        }
    }

    #[inline]
    fn pattern_table(&self) -> PatternTable {
        self.memory.pattern_table()
    }

    #[inline]
    fn name_table(&self, number: NameTableNumber) -> NameTable {
        self.memory.name_table(number)
    }

    fn rendering_enabled(&self) -> bool {
        self.mask.sprites_enabled() || self.mask.background_enabled()
    }
}

pub struct StepEvents {
    vblank_event: VBlankEvent,
    nmi_trigger: bool,
}

impl StepEvents {
    pub fn no_events() -> StepEvents {
        StepEvents {
            vblank_event: VBlankEvent::None,
            nmi_trigger: false,
        }
    }

    pub fn start_vblank() -> StepEvents {
        StepEvents {
            vblank_event: VBlankEvent::Started,
            nmi_trigger: true,
        }
    }

    pub fn stop_vblank() -> StepEvents {
        StepEvents {
            vblank_event: VBlankEvent::Stopped,
            nmi_trigger: false,
        }
    }

    pub fn vblank_event(&self) -> VBlankEvent {
        self.vblank_event
    }

    pub fn nmi_trigger(&self) -> bool {
        self.nmi_trigger
    }
}

#[derive(Clone, Copy)]
pub enum VBlankEvent {
    None,
    Started,
    Stopped,
}
