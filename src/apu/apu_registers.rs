use crate::apu::pulse_channel::PulseChannel;
use crate::apu::triangle_channel::TriangleChannel;
use crate::apu::noise_channel::NoiseChannel;
use crate::apu::dmc::Dmc;
use crate::apu::frame_counter::FrameCounter;

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
        Status {
            dmc_interrupt: false,
            frame_interrupt: false,
            dmc_active: false,
            noise_active: false,
            triangle_active: false,
            pulse_2_active: false,
            pulse_1_active: false,
        }
    }

    pub fn write_status(&mut self, value: u8) {
        self.dmc.enabled      = value & 0b0001_0000 != 0;
        self.noise.enabled    = value & 0b0000_1000 != 0;
        self.triangle.enabled = value & 0b0000_0100 != 0;
        self.pulse_1.enabled  = value & 0b0000_0010 != 0;
        self.pulse_2.enabled  = value & 0b0000_0001 != 0;
    }
}

pub struct Status {
    dmc_interrupt: bool,
    frame_interrupt: bool,
    dmc_active: bool,
    noise_active: bool,
    triangle_active: bool,
    pulse_2_active: bool,
    pulse_1_active: bool,
}
