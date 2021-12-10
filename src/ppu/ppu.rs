use crate::ppu::address::Address;
use crate::ppu::clock::Clock;
use crate::ppu::memory::Memory;
use crate::ppu::name_table::NameTable;
use crate::ppu::name_table_number::NameTableNumber;
use crate::ppu::oam::Oam;
use crate::ppu::pattern_table::PatternTable;
use crate::ppu::palette::palette_table::PaletteTable;
use crate::ppu::palette::palette_index::PaletteIndex;
use crate::ppu::palette::system_palette::SystemPalette;
use crate::ppu::register::ctrl::Ctrl;
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

    screen: Screen,
    system_palette: SystemPalette,

    vram_address: Address,
    next_vram_upper_byte: Option<u8>,
}

impl Ppu {
    pub fn new(system_palette: SystemPalette) -> Ppu {
        Ppu {
            memory: Memory::new(),
            oam: Oam::new(),
            ctrl: Ctrl::new(),
            mask: Mask::new(),

            clock: Clock::new(),

            screen: Screen::new(),
            system_palette,

            vram_address: Address::from_u16(0),
            next_vram_upper_byte: None,
        }
    }

    pub fn clock(&self) -> &Clock {
        &self.clock
    }

    pub fn screen(&self) -> &Screen {
        &self.screen
    }

    pub fn set_ctrl(&mut self, ctrl: Ctrl) {
        self.ctrl = ctrl;
    }

    pub fn set_mask(&mut self, mask: Mask) {
        self.mask = mask;
    }

    pub fn write_vram(&mut self, value: u8) {
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

    pub fn step(&mut self) -> StepEvents {
        self.clock.tick();

        let frame_started = self.clock().is_start_of_frame();
        if frame_started {
            println!("PPU Cycle: {}, Frame: {}", self.clock().total_cycles(), self.clock().frame());
        }
        match self.clock().total_cycles() {
            FIRST_VBLANK_CYCLE | SECOND_VBLANK_CYCLE =>
                return StepEvents::start_vblank(),
            frame if frame < SECOND_VBLANK_CYCLE =>
                return StepEvents::no_events(),
            // The PPU has warmed up, proceed with rendering.
            _ => {},
        }

        if self.clock.cycle() == 0 {
            for tile_number in TileNumber::iter() {
                for row_in_tile in 0..8 {
                    let name_table_number = self.ctrl.name_table_number();
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

        StepEvents::no_events()
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
}

pub struct StepEvents {
    vblank_started: bool,
}

impl StepEvents {
    pub fn no_events() -> StepEvents {
        StepEvents {
            vblank_started: false,
        }
    }

    pub fn start_vblank() -> StepEvents {
        println!("Starting VBlank.");
        StepEvents {
            vblank_started: true,
        }
    }

    pub fn vblank_started(&self) -> bool {
        self.vblank_started
    }
}
