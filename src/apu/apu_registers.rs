use crate::apu::pulse_channel::PulseChannel;
use crate::apu::triangle_channel::TriangleChannel;
use crate::apu::noise_channel::NoiseChannel;
use crate::apu::dmc::Dmc;
use crate::util::bit_util;

#[derive(Default)]
pub struct ApuRegisters {
    pub pulse_1: PulseChannel,
    pub pulse_2: PulseChannel,
    pub triangle: TriangleChannel,
    pub noise: NoiseChannel,
    pub dmc: Dmc,

    step_mode: StepMode,
    suppress_irq: bool,
    frame_irq_pending: bool,
}

impl ApuRegisters {
    pub fn step_mode(&self) -> StepMode {
        self.step_mode
    }

    pub fn read_status(&mut self) -> Status {
        let frame_irq_pending = self.frame_irq_pending;
        self.frame_irq_pending = false;

        Status {
            dmc_interrupt: self.dmc.irq_enabled,
            frame_irq_pending,
            dmc_active: self.dmc.active(),
            noise_active: self.noise.active(),
            triangle_active: self.triangle.active(),
            pulse_2_active: self.pulse_2.active(),
            pulse_1_active: self.pulse_1.active(),
        }
    }

    pub fn write_status_byte(&mut self, value: u8) {
        self.dmc.set_enabled(     value & 0b0001_0000 != 0);
        self.noise.set_enabled(   value & 0b0000_1000 != 0);
        self.triangle.set_enabled(value & 0b0000_0100 != 0);
        self.pulse_2.set_enabled( value & 0b0000_0010 != 0);
        self.pulse_1.set_enabled( value & 0b0000_0001 != 0);
    }

    pub fn write_frame_counter(&mut self, value: u8) {
        let old_step_mode = self.step_mode;
        use StepMode::*;
        self.step_mode = if value & 0b1000_0000 == 0 { FourStep } else { FiveStep };
        self.suppress_irq = value & 0b0100_0000 != 0;

        if self.suppress_irq {
            self.frame_irq_pending = false;
        }

        if old_step_mode == StepMode::FourStep && self.step_mode == StepMode::FiveStep {
            self.pulse_1.length_counter.try_set_to_zero();
            self.pulse_2.length_counter.try_set_to_zero();
            self.triangle.length_counter.try_set_to_zero();
            self.noise.length_counter.try_set_to_zero();
        }
    }

    pub fn decrement_length_counters(&mut self) {
        self.pulse_1.length_counter.decrement_towards_zero();
        self.pulse_2.length_counter.decrement_towards_zero();
        self.triangle.length_counter.decrement_towards_zero();
        self.noise.length_counter.decrement_towards_zero();
    }

    pub fn frame_irq_pending(&self) -> bool {
        self.frame_irq_pending
    }

    pub fn maybe_set_frame_irq_pending(&mut self) {
        if self.step_mode == StepMode::FourStep && !self.suppress_irq {
            self.frame_irq_pending = true;
        }
    }

    pub fn acknowledge_frame_irq(&mut self) {
        self.frame_irq_pending = false;
    }
}

#[derive(Clone, Copy)]
pub struct Status {
    dmc_interrupt: bool,
    frame_irq_pending: bool,
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
                self.frame_irq_pending,
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

#[derive(PartialEq, Clone, Copy, Debug, Default)]
pub enum StepMode {
    #[default]
    FourStep,
    FiveStep,
}

impl StepMode {
    pub const FOUR_STEP_FRAME_LENGTH: u16 = 14915;
    pub const FIVE_STEP_FRAME_LENGTH: u16 = 18641;

    pub const fn frame_length(self) -> u16 {
        match self {
            StepMode::FourStep => StepMode::FOUR_STEP_FRAME_LENGTH,
            StepMode::FiveStep => StepMode::FIVE_STEP_FRAME_LENGTH,
        }
    }
}