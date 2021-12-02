use crate::ppu::address::Address;
use crate::ppu::clock::Clock;
use crate::ppu::memory::Memory;
use crate::ppu::oam::Oam;
use crate::ppu::pattern_table::PatternTable;
use crate::ppu::ppu_registers::PpuRegisters;

const PATTERN_TABLE_START: Address = Address::from_u16(0).unwrap();
const PATTERN_TABLE_SIZE: u16 = 0x2000;

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
        self.clock.tick();
    }

    fn pattern_table(&self) -> PatternTable {
        let slice = self.memory.slice(PATTERN_TABLE_START, PATTERN_TABLE_SIZE);
        PatternTable::new(slice.try_into().unwrap())
    }
}
