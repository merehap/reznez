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
    frame_counter_write_status: FrameCounterWriteStatus,

    // Whenever a quarter or half frame signal occurs, recurrence is suppressed for 2 cycles.
    // This is the basis of apu_test_2, motivation described here:
    // https://forums.nesdev.org/viewtopic.php?t=11174&sid=fe21b7f101cf155ca56eda5287c14c89
    counter_suppression_cycles: u8,

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
            frame_counter_write_status: FrameCounterWriteStatus::Inactive,

            counter_suppression_cycles: 0,

            clock: ApuClock::new(),
        }
    }

    pub fn reset(&mut self) {
        // At reset, $4015 should be cleared
        self.write_status_byte(0b0000_0000);
        // At reset, $4017 should should be rewritten with last value written
        self.frame_counter_write_status = FrameCounterWriteStatus::Initialized;
        self.frame_irq_pending = false;
    }

    pub fn step_mode(&self) -> StepMode {
        self.clock.step_mode
    }

    pub fn clock(&self) -> ApuClock {
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

    // Read 0x4015
    pub fn read_status(&mut self) -> Status {
        if self.frame_irq_pending {
            info!(target: "apuevents", "Status read cleared pending frame IRQ. APU Cycle: {}", self.clock.cycle());
        }

        let status = self.peek_status();
        self.frame_irq_pending = false;
        status
    }

    // Write 0x4015
    pub fn write_status_byte(&mut self, value: u8) {
        info!(target: "apuevents", "APU status write: {value:05b} . APU Cycle: {}", self.clock.cycle());

        self.dmc.set_enabled(     value & 0b0001_0000 != 0);
        self.noise.set_enabled(   value & 0b0000_1000 != 0);
        self.triangle.set_enabled(value & 0b0000_0100 != 0);
        self.pulse_2.set_enabled( value & 0b0000_0010 != 0);
        self.pulse_1.set_enabled( value & 0b0000_0001 != 0);
    }

    // Write 0x4017
    pub fn write_frame_counter(&mut self, value: u8) {
        use StepMode::*;
        self.pending_step_mode = if value & 0b1000_0000 == 0 { FourStep } else { FiveStep };
        self.suppress_irq = value & 0b0100_0000 != 0;
        if self.suppress_irq {
            self.frame_irq_pending = false;
        }

        self.frame_counter_write_status = FrameCounterWriteStatus::Initialized;

        info!(target: "apuevents", "Frame counter write: {:?}, Suppress IRQ: {}, Status: {:?}, APU Cycle: {}",
            self.pending_step_mode, self.suppress_irq, self.frame_counter_write_status, self.clock.cycle());
    }

    pub fn maybe_update_step_mode(&mut self) {
        let apu_cycle = self.clock.cycle();
        if self.counter_suppression_cycles > 0 {
            self.counter_suppression_cycles -= 1;
        }

        use FrameCounterWriteStatus::*;
        match self.frame_counter_write_status {
            Inactive => { /* Do nothing. */ }
            Initialized => {
                info!(target: "apuevents", "APU frame counter: Waiting for APU 'ON' cycle. APU Cycle: {apu_cycle}");
                self.frame_counter_write_status = WaitingForOnCycle;
            }
            WaitingForOnCycle if !self.clock.is_off_cycle() => {
                info!(target: "apuevents", "APU frame counter: Resetting on the next APU cycle. APU Cycle: {apu_cycle}");
                self.frame_counter_write_status = Ready;
            }
            WaitingForOnCycle => {
                info!(target: "apuevents", "APU frame counter: Still waiting for APU 'ON' cycle. APU Cycle: {apu_cycle}");
            }
            Ready => {
                info!(
                    target: "apuevents",
                    "APU frame counter: Resetting APU cycle and setting step mode: {:?}. Skipped APU Cycle: {apu_cycle}",
                    self.pending_step_mode,
                );
                self.clock.reset();
                self.clock.step_mode = self.pending_step_mode;
                if self.clock.step_mode == StepMode::FiveStep && self.counter_suppression_cycles == 0 {
                    self.decrement_length_counters();
                    self.counter_suppression_cycles = 2;
                }

                self.frame_counter_write_status = Inactive;
            }
        }
    }

    pub fn on_cycle_step(&mut self) {
        self.pulse_1.on_cycle_step();
        self.pulse_2.on_cycle_step();
        self.triangle.on_cycle_step();
        self.noise.on_cycle_step();
        self.dmc.on_cycle_step();
    }

    pub fn off_cycle_step(&mut self) {
        self.triangle.off_cycle_step();
    }

    pub fn maybe_decrement_counters(&mut self) {
        const FIRST_STEP  : u16 = 2 * 3728 + 1;
        const SECOND_STEP : u16 = 2 * 7456 + 1;
        const THIRD_STEP  : u16 = 2 * 11185 + 1;
        const FOURTH_STEP : u16 = 2 * 14914 + 1;
        const FIFTH_STEP  : u16 = 2 * 18640 + 1;

        let cycle = self.clock.cycle();
        match self.clock.step_mode {
            StepMode::FourStep => assert!(cycle < StepMode::FOUR_STEP_FRAME_LENGTH),
            StepMode::FiveStep => assert!(cycle < StepMode::FIVE_STEP_FRAME_LENGTH),
        }

        match cycle {
            FIRST_STEP => {
                self.triangle.decrement_linear_counter();
                self.counter_suppression_cycles = 2;
            }
            SECOND_STEP => {
                self.triangle.decrement_linear_counter();
                self.decrement_length_counters();
                self.counter_suppression_cycles = 2;
            }
            THIRD_STEP => {
                self.triangle.decrement_linear_counter();
                self.counter_suppression_cycles = 2;
            }
            FOURTH_STEP if self.clock.step_mode == StepMode::FourStep => {
                self.triangle.decrement_linear_counter();
                self.decrement_length_counters();
                self.counter_suppression_cycles = 2;
            }
            FIFTH_STEP if self.clock.step_mode == StepMode::FiveStep => {
                self.triangle.decrement_linear_counter();
                self.decrement_length_counters();
                self.counter_suppression_cycles = 2;
            }
            _ => { /* Do nothing. */ }
        }
    }

    fn decrement_length_counters(&mut self) {
        let p1 = self.pulse_1.length_counter.decrement_towards_zero();
        let p2 = self.pulse_2.length_counter.decrement_towards_zero();
        let t = self.triangle.length_counter.decrement_towards_zero();
        let n = self.noise.length_counter.decrement_towards_zero();

        info!(target: "apuevents", "Decremented length counters. P1: {}, P2: {}, T: {}, N: {}. APU Cycle: {}",
            p1, p2, t, n, self.clock.cycle(),
        );
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

        let cycle = self.clock.cycle();
        let is_non_skip_first_cycle = cycle == 0 && !self.clock.is_forced_reset_cycle();
        let is_last_cycle = cycle == StepMode::FOUR_STEP_FRAME_LENGTH - 1 || cycle == StepMode::FOUR_STEP_FRAME_LENGTH - 2;
        let is_irq_cycle = is_non_skip_first_cycle || is_last_cycle;

        if is_irq_cycle {
            info!(target: "apuevents", "Frame IRQ pending. APU Cycle: {}", self.clock.cycle());
            self.frame_irq_pending = true;
        }
    }

    pub fn acknowledge_frame_irq(&mut self) {
        info!(target: "apuevents", "Frame IRQ acknowledged. APU Cycle: {}", self.clock.cycle());
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
    pub const FOUR_STEP_FRAME_LENGTH: u16 = 2 * 14915;
    pub const FIVE_STEP_FRAME_LENGTH: u16 = 2 * 18641;

    pub const fn frame_length(self) -> u16 {
        match self {
            StepMode::FourStep => StepMode::FOUR_STEP_FRAME_LENGTH,
            StepMode::FiveStep => StepMode::FIVE_STEP_FRAME_LENGTH,
        }
    }
}

#[derive(Clone, Copy)]
pub struct ApuClock {
    cycle: u64,
    step_mode: StepMode,
}

impl ApuClock {
    pub fn new() -> Self {
        Self {
            cycle: 0,
            step_mode: StepMode::FourStep,
        }
    }

    pub fn increment(&mut self) {
        self.cycle += 1;
    }

    pub fn reset(&mut self) {
        self.cycle = 0;
    }

    pub fn is_off_cycle(self) -> bool {
        self.cycle % 2 == 0
    }

    pub fn cycle(self) -> u16 {
        u16::try_from(self.cycle % u64::from(self.step_mode.frame_length())).unwrap()
    }

    pub fn raw_cycle(self) -> u64 {
        // FIXME: Remove the "/ 2" and fix this on the caller's side.
        self.cycle / 2
    }

    pub fn is_forced_reset_cycle(&self) -> bool {
        self.cycle == 0
    }
}


#[derive(PartialEq, Eq, Debug)]
enum FrameCounterWriteStatus {
    Inactive,
    Initialized,
    WaitingForOnCycle,
    Ready,
}
