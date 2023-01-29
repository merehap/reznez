use rodio::source::Source;

use crate::apu::apu_registers::ApuRegisters;

pub struct Apu {
}

impl Apu {
    pub fn new() -> Apu {
        Apu {}
    }

    pub fn step_triangle_channel_only(&self, regs: &mut ApuRegisters) {
        //regs.triangle.step();
    }

    pub fn step(&self, regs: &mut ApuRegisters) {
        regs.pulse_1.step();
        //regs.pulse_2.step();
        //regs.triangle.step();
    }
}
