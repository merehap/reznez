use crate::ppu::clock::Clock;
use crate::ppu::memory::Memory;
use crate::ppu::name_table::NameTable;
use crate::ppu::name_table_number::NameTableNumber;
use crate::ppu::oam::Oam;
use crate::ppu::pattern_table::PatternTable;
use crate::ppu::palette::palette_table::PaletteTable;
use crate::ppu::tile_number::TileNumber;
use crate::ppu::ppu_registers::PpuRegisters;

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
                    let name_table_number = regs.name_table_number();
                    let (tile_index, _palette_table_index) =
                        self.name_table(name_table_number).tile_entry_at(tile_number);
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
        self.memory.pattern_table()
    }

    fn name_table(&self, number: NameTableNumber) -> NameTable {
        self.memory.name_table(number)
    }

    fn palette_table(&self) -> PaletteTable {
        self.memory.palette_table()
    }
}
