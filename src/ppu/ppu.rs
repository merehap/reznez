use crate::ppu::address::Address;
use crate::ppu::clock::Clock;
use crate::ppu::memory::Memory;
use crate::ppu::name_table::NameTable;
use crate::ppu::oam::Oam;
use crate::ppu::pattern_table::PatternTable;
use crate::ppu::palette::palette_table::PaletteTable;
use crate::ppu::tile_number::TileNumber;
use crate::ppu::ppu_registers::PpuRegisters;

const PATTERN_TABLE_START: Address = Address::from_u16(0).unwrap();
const PATTERN_TABLE_SIZE: u16 = 0x2000;

const NAME_TABLE_START: Address = Address::from_u16(0x2000).unwrap();
const NAME_TABLE_SIZE: u16 = 0x400;
const NAME_TABLE_INDEXES: [Address; 4] =
    [
        NAME_TABLE_START.advance(0 * NAME_TABLE_SIZE),
        NAME_TABLE_START.advance(1 * NAME_TABLE_SIZE),
        NAME_TABLE_START.advance(2 * NAME_TABLE_SIZE),
        NAME_TABLE_START.advance(3 * NAME_TABLE_SIZE),
    ];

const PALETTE_TABLE_START: Address = Address::from_u16(0x3F00).unwrap();
const PALETTE_TABLE_SIZE: u16 = 0x20;

pub struct Ppu {
    memory: Memory,
    oam: Oam,
    clock: Clock,
}

impl Ppu {
    pub fn startup() -> Ppu {
        Ppu {
            memory: Memory::new(),
            oam: Oam::new(),
            clock: Clock::new(),
        }
    }

    pub fn step(&mut self, regs: PpuRegisters<'_>) {
        if self.clock.cycle() == 0 {
            for tile_number in TileNumber::iter() {
                for row_in_tile in 0..8 {
                    let name_table_number = regs.name_table_number() as usize;
                    let (tile_index, _palette_table_index) =
                        self.name_tables()[name_table_number].tile_entry_at(tile_number);
                    let _tile_sliver = self.pattern_table().tile_sliver_at(
                        regs.background_table_side(),
                        tile_index,
                        row_in_tile,
                        );

                }
            }
        }

        self.clock.tick();
    }

    fn pattern_table(&self) -> PatternTable {
        let slice = self.memory.slice(PATTERN_TABLE_START, PATTERN_TABLE_SIZE);
        PatternTable::new(slice.try_into().unwrap())
    }

    fn name_tables(&self) -> [NameTable; 4] {
        NAME_TABLE_INDEXES.map(|index| {
            let slice = self.memory.slice(index, NAME_TABLE_SIZE);
            NameTable::new(slice.try_into().unwrap())
        })
    }

    fn palette_table(&self) -> PaletteTable {
        let slice = self.memory.slice(PALETTE_TABLE_START, PALETTE_TABLE_SIZE);
        PaletteTable::new(slice.try_into().unwrap())
    }
}
