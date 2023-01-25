use crate::apu::pulse_channel::PulseChannel;
use crate::apu::triangle_channel::TriangleChannel;
use crate::apu::noise_channel::NoiseChannel;
use crate::apu::dmc::Dmc;
use crate::apu::frame_counter::FrameCounter;
use crate::util::bit_util;

#[derive(Default)]
pub struct ApuRegisters {
    pub pulse_1: PulseChannel,
    pub pulse_2: PulseChannel,
    pub triangle: TriangleChannel,
    pub noise: NoiseChannel,
    pub dmc: Dmc,
    pub frame_counter: FrameCounter,
}

impl ApuRegisters {
    pub fn read_status(&mut self) -> Status {
        let frame_interrupt = self.frame_counter.frame_interrupt;
        self.frame_counter.frame_interrupt = false;

        Status {
            dmc_interrupt: self.dmc.irq_enabled,
            frame_interrupt,
            dmc_active: self.dmc.active(),
            noise_active: self.noise.active(),
            triangle_active: self.triangle.active(),
            pulse_2_active: self.pulse_2.active(),
            pulse_1_active: self.pulse_1.active(),
        }
    }

    pub fn write_status_byte(&mut self, value: u8) {
        self.dmc.enable_or_disable(value & 0b0001_0000 != 0);
        self.noise.enable_or_disable(value & 0b0000_1000 != 0);
        self.triangle.enable_or_disable(value & 0b0000_0100 != 0);
        self.pulse_1.enable_or_disable(value & 0b0000_0010 != 0);
        self.pulse_2.enable_or_disable(value & 0b0000_0001 != 0);
    }
}

#[derive(Clone, Copy)]
pub struct Status {
    dmc_interrupt: bool,
    frame_interrupt: bool,
    dmc_active: bool,
    noise_active: bool,
    triangle_active: bool,
    pulse_2_active: bool,
    pulse_1_active: bool,
}

impl Status {
    pub fn to_u8(self) -> u8 {
        bit_util::pack_bools(
            [
                self.dmc_interrupt,
                self.frame_interrupt,
                false,
                self.dmc_active,
                self.noise_active,
                self.triangle_active,
                self.pulse_2_active,
                self.pulse_1_active,
            ]
        )
    }
}
