use crate::ppu::memory::Memory;
use crate::ppu::ppu_registers::PpuRegisters;

pub struct Ppu {
    memory: Memory,
}

impl Ppu {
    pub fn startup() -> Ppu {
        Ppu {
            memory: Memory::new(),
        }
    }

    pub fn step(&mut self, _ppu_registers: PpuRegisters<'_>) {

    }
}
