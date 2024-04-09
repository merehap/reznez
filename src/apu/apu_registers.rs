use log::info;

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

    pending_step_mode: StepMode,
    step_mode: StepMode,
    suppress_irq: bool,
    frame_irq_pending: bool,
    write_delay: Option<u8>,

    off_cycle: bool,
    cycle: u64,

    // Reloading the cycle counter must not cause a frame IRQ on cycle 0.
    pub is_frame_irq_skip_cycle: bool,
}

impl ApuRegisters {
    pub fn new() -> ApuRegisters {
        let mut result = ApuRegisters::default();
        result.is_frame_irq_skip_cycle = true;
        result
    }

    pub fn step_mode(&self) -> StepMode {
        self.step_mode
    }

    pub fn cycle(&self) -> u64 {
        self.cycle
    }

    pub fn off_cycle(&mut self) {
        self.off_cycle = true;
    }

    pub fn on_cycle(&mut self) {
        self.off_cycle = false;
    }

    pub fn increment_cycle(&mut self) {
        self.cycle += 1;
    }

    pub fn peek_status(&self) -> Status {
        Status {
            dmc_interrupt: self.dmc.irq_pending,
            frame_irq_pending: self.frame_irq_pending,
            dmc_active: self.dmc.active(),
            noise_active: self.noise.active(),
            triangle_active: self.triangle.active(),
            pulse_2_active: self.pulse_2.active(),
            pulse_1_active: self.pulse_1.active(),
        }
    }

    pub fn read_status(&mut self) -> Status {
        if self.frame_irq_pending {
            info!(target: "apuevents", "Status read cleared pending frame IRQ.");
        }

        let status = self.peek_status();
        self.frame_irq_pending = false;
        status
    }

    pub fn write_status_byte(&mut self, value: u8) {
        info!(target: "apuevents", "APU status write: {value:05b}");

        self.dmc.set_enabled(     value & 0b0001_0000 != 0);
        self.noise.set_enabled(   value & 0b0000_1000 != 0);
        self.triangle.set_enabled(value & 0b0000_0100 != 0);
        self.pulse_2.set_enabled( value & 0b0000_0010 != 0);
        self.pulse_1.set_enabled( value & 0b0000_0001 != 0);
    }

    pub fn write_frame_counter(&mut self, value: u8) {
        use StepMode::*;
        self.pending_step_mode = if value & 0b1000_0000 == 0 { FourStep } else { FiveStep };
        self.suppress_irq = value & 0b0100_0000 != 0;
        if self.suppress_irq {
            self.frame_irq_pending = false;
        }

        let write_delay = if self.off_cycle { 4 } else { 3 };
        info!(target: "apuevents", "Frame counter write: {:?}, Suppress IRQ: {}, Write delay: {}",
            self.pending_step_mode, self.suppress_irq, write_delay);

        self.write_delay = Some(write_delay);
    }

    pub fn maybe_update_step_mode(&mut self) {
        if let Some(write_delay) = self.write_delay {
            if write_delay == 1 {
                info!(target: "apuevents", "Resetting APU cycle and setting step mode.");
                self.is_frame_irq_skip_cycle = true;
                self.cycle = 0;
                self.write_delay = None;
                self.step_mode = self.pending_step_mode;
                if self.step_mode == StepMode::FiveStep {
                    self.decrement_length_counters();
                }
            } else {
                self.write_delay = Some(write_delay - 1);
            }
        } else if !self.off_cycle {
            self.increment_cycle();
        }
    }

    pub fn decrement_length_counters(&mut self) {
        info!(target: "apuevents", "Decrementing length counters.");
        self.pulse_1.length_counter.decrement_towards_zero();
        self.pulse_2.length_counter.decrement_towards_zero();
        self.triangle.length_counter.decrement_towards_zero();
        self.noise.length_counter.decrement_towards_zero();
    }

    pub fn frame_irq_pending(&self) -> bool {
        self.frame_irq_pending
    }

    pub fn dmc_irq_pending(&self) -> bool {
        self.dmc.irq_pending
    }

    pub fn maybe_set_frame_irq_pending(&mut self) {
        if self.step_mode == StepMode::FourStep && !self.suppress_irq {
            info!(target: "apuevents", "Frame IRQ pending.");
            self.frame_irq_pending = true;
        }
    }

    pub fn acknowledge_frame_irq(&mut self) {
        info!(target: "apuevents", "Frame IRQ acknowledged.");
        self.frame_irq_pending = false;
    }
}

#[derive(Clone, Copy, Debug)]
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
