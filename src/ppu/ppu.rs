use crate::ppu::ppu_registers::PpuRegisters;

pub struct Ppu {

}

impl Ppu {
    pub fn startup() -> Ppu {
        Ppu {}
    }

    pub fn step(&mut self, _ppu_registers: PpuRegisters<'_>) {

    }
}
