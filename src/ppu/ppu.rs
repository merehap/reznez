use crate::ppu::address::Address;
use crate::ppu::clock::Clock;
use crate::ppu::memory::Memory;
use crate::ppu::name_table::NameTable;
use crate::ppu::name_table_mirroring::NameTableMirroring;
use crate::ppu::name_table_number::NameTableNumber;
use crate::ppu::oam::Oam;
use crate::ppu::pattern_table::PatternTable;
use crate::ppu::palette::palette_table::PaletteTable;
use crate::ppu::palette::palette_index::PaletteIndex;
use crate::ppu::palette::system_palette::SystemPalette;
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
    name_table_mirroring: NameTableMirroring,

    clock: Clock,

    screen: Screen,
    system_palette: SystemPalette,

    is_nmi_period: bool,
    vram_address: Address,
    next_vram_upper_byte: Option<u8>,
    oam_address: u8,
}

impl Ppu {
    pub fn new(
        name_table_mirroring: NameTableMirroring,
        system_palette: SystemPalette,
        ) -> Ppu {

        Ppu {
            memory: Memory::new(name_table_mirroring),
            oam: Oam::new(),
            ctrl: Ctrl::new(),
            mask: Mask::new(),
            name_table_mirroring,

            clock: Clock::new(),

            screen: Screen::new(),
            system_palette,

            is_nmi_period: false,
            vram_address: Address::from_u16(0),
            next_vram_upper_byte: None,
            oam_address: 0,
        }
    }

    pub fn clock(&self) -> &Clock {
        &self.clock
    }

    pub fn screen(&self) -> &Screen {
        &self.screen
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

    pub fn step(&mut self) -> StepEvents {
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
        } else if self.clock.cycle() == 0 {
            self.render();
            step_event = StepEvents::no_events();
        } else {
            step_event = StepEvents::no_events();
        }

        self.clock.tick(self.rendering_enabled());
        step_event
    }

    fn render(&mut self) {
        let name_table_number = self.ctrl.name_table_number();
        println!("Nametable #{:?}: {}", name_table_number, self.name_table(name_table_number));

        for tile_number in TileNumber::iter() {
            for row_in_tile in 0..8 {
                let (tile_index, palette_table_index) =
                    self.name_table(name_table_number).tile_entry_at(tile_number);
                let palette =
                    self.palette_table().background_palette(palette_table_index);
                let tile_sliver: [Option<PaletteIndex>; 8] =
                    self.pattern_table().tile_sliver_at(
                        self.ctrl.background_table_side(),
                        tile_index,
                        row_in_tile,
                        );
                let pixel_row = 8 * tile_number.row() + row_in_tile;
                for (column_in_tile, palette_index) in tile_sliver.iter().enumerate() {
                    let pixel_column =
                        8 * tile_number.column() + column_in_tile as u8;
                    if let Some(palette_index) = palette_index {
                        let rgb = self.system_palette.lookup_rgb(palette[*palette_index]);
                        self.screen.set_pixel(pixel_column, pixel_row, rgb);
                    }
                }
            }
        }
    }

    fn pattern_table(&self) -> PatternTable {
        self.memory.pattern_table()
    }

    fn name_table(&self, number: NameTableNumber) -> NameTable {
        self.memory.name_table(number)
    }

    fn palette_table(&self) -> PaletteTable {
        self.memory.palette_table()
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
