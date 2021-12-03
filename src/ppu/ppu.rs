use crate::ppu::address::Address;
use crate::ppu::clock::Clock;
use crate::ppu::memory::Memory;
use crate::ppu::name_table::NameTable;
use crate::ppu::oam::Oam;
use crate::ppu::pattern_table::PatternTable;
use crate::ppu::palette::palette_table::PaletteTable;
use crate::ppu::ppu_registers::PpuRegisters;

const PATTERN_TABLE_START: Address = Address::from_u16(0).unwrap();
const PATTERN_TABLE_SIZE: u16 = 0x2000;

const NAME_TABLE_START: Address = Address::from_u16(0x2000).unwrap();
const NAME_TABLE_SIZE: u16 = 0x400;

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

    pub fn step(&mut self, _ppu_registers: PpuRegisters<'_>) {
        if self.clock.cycle() == 0 {

        }

        self.clock.tick();
    }

    fn pattern_table(&self) -> PatternTable {
        let slice = self.memory.slice(PATTERN_TABLE_START, PATTERN_TABLE_SIZE);
        PatternTable::new(slice.try_into().unwrap())
    }

    fn first_name_table(&self) -> NameTable {
        let slice = self.memory.slice(NAME_TABLE_START, NAME_TABLE_SIZE);
        NameTable::new(slice.try_into().unwrap())
    }

    fn palette_table(&self) -> PaletteTable {
        let slice = self.memory.slice(PALETTE_TABLE_START, PALETTE_TABLE_SIZE);
        PaletteTable::new(slice.try_into().unwrap())
    }
}
