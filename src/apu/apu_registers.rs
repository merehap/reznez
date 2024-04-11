use log::info;

use crate::apu::pulse_channel::PulseChannel;
use crate::apu::triangle_channel::TriangleChannel;
use crate::apu::noise_channel::NoiseChannel;
use crate::apu::dmc::Dmc;
use crate::util::bit_util;

pub struct ApuRegisters {
    pub pulse_1: PulseChannel,
    pub pulse_2: PulseChannel,
    pub triangle: TriangleChannel,
    pub noise: NoiseChannel,
    pub dmc: Dmc,

    pending_step_mode: StepMode,
    suppress_irq: bool,
    frame_irq_pending: bool,
    write_delay: Option<u8>,

    clock: ApuClock,
}

impl ApuRegisters {
    pub fn new() -> ApuRegisters {
        ApuRegisters {
            pulse_1: PulseChannel::default(),
            pulse_2: PulseChannel::default(),
            triangle: TriangleChannel::default(),
            noise: NoiseChannel::default(),
            dmc: Dmc::default(),

            pending_step_mode: StepMode::FourStep,
            suppress_irq: false,
            frame_irq_pending: false,
            write_delay: None,

            clock: ApuClock::new(),
        }
    }

    pub fn step_mode(&self) -> StepMode {
        self.clock.step_mode
    }

    pub fn clock(&self) -> ApuClock{
        self.clock
    }

    pub fn clock_mut(&mut self) -> &mut ApuClock {
        &mut self.clock
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

        let write_delay = if self.clock.is_off_cycle { 4 } else { 3 };
        info!(target: "apuevents", "Frame counter write: {:?}, Suppress IRQ: {}, Write delay: {}",
            self.pending_step_mode, self.suppress_irq, write_delay);

        self.write_delay = Some(write_delay);
    }

    pub fn maybe_update_step_mode(&mut self) {
        if let Some(write_delay) = self.write_delay {
            if write_delay == 1 {
                info!(target: "apuevents", "Resetting APU cycle and setting step mode.");
                self.clock.reset();
                self.write_delay = None;
                self.clock.step_mode = self.pending_step_mode;
                if self.clock.step_mode == StepMode::FiveStep {
                    self.decrement_length_counters();
                }
            } else {
                self.write_delay = Some(write_delay - 1);
            }
        } else {
            self.clock.increment();
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
        if self.suppress_irq || self.clock.step_mode != StepMode::FourStep {
            return;
        }

        let cycle = self.clock.to_u16();
        let is_non_skip_first_cycle = cycle == 0 && self.clock.to_raw() != 0 && self.clock.is_off_cycle;
        let is_last_cycle = self.clock.to_u16() == StepMode::FOUR_STEP_FRAME_LENGTH - 1;
        let is_irq_cycle = is_non_skip_first_cycle || is_last_cycle;

        if is_irq_cycle {
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

#[derive(Clone, Copy)]
pub struct ApuClock {
    cycle: i64,
    is_off_cycle: bool,
    step_mode: StepMode,
}

impl ApuClock {
    pub fn new() -> Self {
        Self {
            cycle: 0,
            is_off_cycle: false,
            step_mode: StepMode::FourStep,
        }
    }

    pub fn toggle(&mut self) {
        self.is_off_cycle = !self.is_off_cycle;
    }

    pub fn increment(&mut self) {
        if !self.is_off_cycle {
            self.cycle += 1;
        }
    }

    pub fn reset(&mut self) {
        self.cycle = 0;
        self.is_off_cycle = true;
    }

    pub fn is_off_cycle(self) -> bool {
        self.is_off_cycle
    }

    pub fn to_u16(self) -> u16 {
        u16::try_from(self.cycle % i64::from(self.step_mode.frame_length())).unwrap()
    }

    pub fn to_raw(self) -> i64 {
        self.cycle
    }
}
