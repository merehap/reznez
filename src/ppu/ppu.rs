use std::ops::Add;
use std::time::{Duration, SystemTime};
use std::thread;

use crate::ppu::address::Address;
use crate::ppu::clock::Clock;
use crate::ppu::memory::Memory;
use crate::ppu::name_table::background_tile_index::BackgroundTileIndex;
use crate::ppu::name_table::name_table::NameTable;
use crate::ppu::name_table::name_table_number::NameTableNumber;
use crate::ppu::oam::Oam;
use crate::ppu::palette::palette_table::PaletteTable;
use crate::ppu::pattern_table::PatternTable;
use crate::ppu::register::ctrl::{Ctrl, VBlankNmi};
use crate::ppu::register::mask::Mask;
use crate::ppu::screen::Screen;

const FIRST_VBLANK_CYCLE: u64 = 3 * 27384;
const SECOND_VBLANK_CYCLE: u64 = 3 * 57165;

const NTSC_FRAME_RATE: f64 = 60.0988;
const NTSC_TIME_PER_FRAME: Duration =
    Duration::from_nanos((1_000_000_000.0 / NTSC_FRAME_RATE) as u64);

pub struct Ppu {
    memory: Memory,
    oam: Oam,

    clock: Clock,
    frame_end_time: SystemTime,

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

            clock: Clock::new(),
            frame_end_time: SystemTime::now(),

            is_nmi_period: false,
            vram_address: Address::from_u16(0),
            next_vram_upper_byte: None,
            oam_address: 0,
        }
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

    pub fn write_vram(&mut self, ctrl: Ctrl, value: u8) {
        println!("Writing to VRAM Address: [{}]={}", self.vram_address, value);
        self.memory[self.vram_address] = value;
        let increment = ctrl.vram_address_increment() as u8;
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
        self.is_nmi_period && ctrl.vblank_nmi() == VBlankNmi::On
    }

    pub fn step(&mut self, ctrl: Ctrl, mask: Mask, screen: &mut Screen) -> StepEvents {
        let frame_started = self.clock().is_first_cycle_of_frame();
        if frame_started {
            self.frame_end_time = SystemTime::now().add(NTSC_TIME_PER_FRAME);
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
        } else if self.clock.scanline() == 1 && self.clock.cycle() == 1 {
            if mask.background_enabled() {
                self.render_background(ctrl, screen);
            }

            if mask.sprites_enabled() {
                self.render_sprites(ctrl, screen);
            }
            step_event = StepEvents::no_events();
        } else {
            step_event = StepEvents::no_events();
        }

        let frame_ended = self.clock().is_last_cycle_of_frame();
        if frame_ended {
            if let Ok(duration) = self.frame_end_time.duration_since(SystemTime::now()) {
                thread::sleep(duration)
            }
        }

        self.clock.tick(self.rendering_enabled(mask));
        step_event
    }

    fn render_background(&mut self, ctrl: Ctrl, screen: &mut Screen) {
        let palette_table = self.memory.palette_table();
        screen.set_universal_background_rgb(palette_table.universal_background_rgb());

        let name_table_number = ctrl.name_table_number();
        let background_table_side = ctrl.background_table_side();
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
                    screen.background_tile_sliver(pixel_column, start_row + row_in_tile),
                    );
            }
        }
    }

    fn render_sprites(&mut self, ctrl: Ctrl, screen: &mut Screen) {
        screen.clear_sprite_buffer();

        let palette_table = self.memory.palette_table();
        let sprite_table_side = ctrl.sprite_table_side();
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

                let mut sliver = screen.sprites_tile_sliver(x, y);
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

    fn rendering_enabled(&self, mask: Mask) -> bool {
        mask.sprites_enabled() || mask.background_enabled()
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
