use crate::ppu::memory::Memory;
use crate::ppu::oam::Oam;
use crate::ppu::ppu_registers::PpuRegisters;

pub struct Ppu {
    memory: Memory,
    oam: Oam,
}

impl Ppu {
    pub fn startup() -> Ppu {
        Ppu {
            memory: Memory::new(),
            oam: Oam::new(),
        }
    }

    pub fn step(&mut self, _ppu_registers: PpuRegisters<'_>) {

    }
}
