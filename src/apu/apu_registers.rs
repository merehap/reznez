use log::info;
use splitbits::splitbits;

use crate::apu::apu_clock::{ApuClock, CycleParity, StepMode};
use crate::apu::sweep::NegateBehavior;
use crate::apu::pulse_channel::PulseChannel;
use crate::apu::triangle_channel::TriangleChannel;
use crate::apu::noise_channel::NoiseChannel;
use crate::apu::dmc::Dmc;
use crate::cpu::dmc_dma::DmcDma;
use crate::memory::cpu::cpu_pinout::CpuPinout;
use crate::util::bit_util;

pub struct ApuRegisters {
    pub pulse_1: PulseChannel<{NegateBehavior::OnesComplement}>,
    pub pulse_2: PulseChannel<{NegateBehavior::TwosComplement}>,
    pub triangle: TriangleChannel,
    pub noise: NoiseChannel,
    pub dmc: Dmc,

    pending_step_mode: StepMode,
    dmc_enabled: bool,
    frame_irq_status: bool,
    suppress_frame_irq: bool,
    should_acknowledge_frame_irq: bool,
    clock_reset_status: ClockResetStatus,

    // Whenever a quarter or half frame signal occurs, recurrence is suppressed for 2 cycles.
    // This is the basis of apu_test_2, motivation described here:
    // https://forums.nesdev.org/viewtopic.php?t=11174&sid=fe21b7f101cf155ca56eda5287c14c89
    counter_suppression_cycles: u8,
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
            dmc_enabled: false,
            frame_irq_status: false,
            suppress_frame_irq: false,
            should_acknowledge_frame_irq: false,
            clock_reset_status: ClockResetStatus::Inactive,

            counter_suppression_cycles: 0,
        }
    }

    pub fn reset(&mut self, clock: &ApuClock, cpu_pinout: &mut CpuPinout) {
        // At reset, $4015 should be cleared
        self.disable_channels();
        cpu_pinout.acknowledge_dmc_irq();
        // At reset, $4017 should should be rewritten with last value written
        self.clock_reset_status.initialize();
        info!(target: "apuevents", "Frame IRQ acknowledged by RESET. APU Cycle: {}", clock.cpu_cycle());
        cpu_pinout.acknowledge_frame_irq();
        self.frame_irq_status = false;
    }

    pub fn clock_reset_status(&self) -> ClockResetStatus {
        self.clock_reset_status
    }

    pub fn peek_status(&self, cpu_pinout: &CpuPinout, dma: &DmcDma) -> Status {
        Status {
            dmc_interrupt: cpu_pinout.dmc_irq_asserted(),
            frame_irq_status: self.frame_irq_status,
            dmc_active: self.dmc_enabled && dma.enabled(),
            noise_active: self.noise.active(),
            triangle_active: self.triangle.active(),
            pulse_2_active: self.pulse_2.active(),
            pulse_1_active: self.pulse_1.active(),
        }
    }

    // Read 0x4015
    pub fn read_status(&mut self, clock: &ApuClock, cpu_pinout: &CpuPinout, dma: &DmcDma) -> Status {
        if cpu_pinout.frame_irq_asserted() {
            info!(target: "apuevents", "Frame IRQ flag will be cleared during the next GET cycle due to APUStatus read. APU Cycle: {}", clock.cpu_cycle());
        }

        let status = self.peek_status(cpu_pinout, dma);
        // Clearing Frame IRQ pending must be delayed until the next GET cycle.
        self.should_acknowledge_frame_irq = true;
        status
    }

    // Write 0x4015
    pub fn write_status_byte(&mut self, clock: &ApuClock, cpu_pinout: &mut CpuPinout, dmc_dma: &mut DmcDma) {
        let value = cpu_pinout.data_bus;
        info!(target: "apuevents", "APU status write: {value:05b} . APU Cycle: {}", clock.cpu_cycle());

        let enabled_channels = splitbits!(value, "...dntqp");
        self.dmc.set_enabled(cpu_pinout, dmc_dma, clock.cycle_parity(), enabled_channels.d);
        // This applies immediately, unlike the similar flag within DMC.
        self.dmc_enabled = enabled_channels.d;

        self.noise.set_enabled(enabled_channels.n);
        self.triangle.set_enabled(enabled_channels.t);
        self.pulse_2.set_enabled(enabled_channels.q);
        self.pulse_1.set_enabled(enabled_channels.p);
    }

    // Upon RESET
    pub fn disable_channels(&mut self) {
        self.noise.set_enabled(false);
        self.triangle.set_enabled(false);
        self.pulse_2.set_enabled(false);
        self.pulse_1.set_enabled(false);
    }

    // Write 0x4017
    pub fn write_frame_counter(&mut self, clock: &ApuClock, cpu_pinout: &mut CpuPinout) {
        let value = cpu_pinout.data_bus;
        use StepMode::*;
        self.pending_step_mode = if value & 0b1000_0000 == 0 { FourStep } else { FiveStep };
        self.suppress_frame_irq = value & 0b0100_0000 != 0;
        if self.suppress_frame_irq {
            info!(target: "apuevents", "Frame IRQ acknowledged by Frame Counter write. APU Cycle: {}", clock.cpu_cycle());
            cpu_pinout.acknowledge_frame_irq();
            self.frame_irq_status = false;
        }

        self.clock_reset_status.initialize();

        info!(target: "apuevents", "Frame counter write: {:?}, Suppress IRQ: {}, Status: {:?}, APU Cycle: {}",
            self.pending_step_mode, self.suppress_frame_irq, self.clock_reset_status, clock.cpu_cycle());
    }

    pub fn tick(&mut self, clock: &mut ApuClock, cpu_pinout: &mut CpuPinout, dmc_dma: &mut DmcDma) {
        if self.counter_suppression_cycles > 0 {
            self.counter_suppression_cycles -= 1;
        }

        let parity = clock.cycle_parity();
        if clock.cycle_parity() == CycleParity::Get {
            self.maybe_update_step_mode(clock);

            if self.should_acknowledge_frame_irq {
                info!(target: "apuevents", "Frame IRQ acknowledged by APUSTATUS read. APU Cycle: {}", clock.cpu_cycle());
                cpu_pinout.acknowledge_frame_irq();
                self.frame_irq_status = false;
                self.should_acknowledge_frame_irq = false;
            }
        }

        self.maybe_set_frame_irq_pending(clock, cpu_pinout);
        if parity == CycleParity::Put {
            self.maybe_decrement_counters(clock);
        }

        self.apply_length_counter_pending_values();
        self.pulse_1.tick(parity);
        self.pulse_2.tick(parity);
        self.triangle.tick();
        self.noise.tick(parity);
        self.dmc.tick(dmc_dma);
    }

    fn maybe_update_step_mode(&mut self, clock: &mut ApuClock) {
        let apu_cycle = clock.cpu_cycle();
        let is_ready = self.clock_reset_status.tick();
        if is_ready {
            info!(
                target: "apuevents",
                "APU frame counter: Resetting APU cycle and setting step mode: {:?}. Skipped APU Cycle: {apu_cycle}",
                self.pending_step_mode,
            );
            clock.reset();
            clock.set_step_mode(self.pending_step_mode);
            if clock.step_mode() == StepMode::FiveStep && self.counter_suppression_cycles == 0 {
                self.decrement_length_counters(clock);
                self.counter_suppression_cycles = 2;
            }
        }
    }

    fn maybe_decrement_counters(&mut self, clock: &ApuClock) {
        const FIRST_STEP  : u16 = 3728;
        const SECOND_STEP : u16 = 7456;
        const THIRD_STEP  : u16 = 11185;
        const FOURTH_STEP : u16 = 14914;
        const FIFTH_STEP  : u16 = 18640;

        match clock.apu_cycle() {
            FIRST_STEP => {
                self.tick_envelopes();
                self.triangle.decrement_linear_counter();
                self.counter_suppression_cycles = 2;
            }
            SECOND_STEP => {
                self.tick_envelopes();
                self.triangle.decrement_linear_counter();
                self.decrement_length_counters(clock);
                self.counter_suppression_cycles = 2;
            }
            THIRD_STEP => {
                self.tick_envelopes();
                self.triangle.decrement_linear_counter();
                self.counter_suppression_cycles = 2;
            }
            FOURTH_STEP if clock.step_mode() == StepMode::FourStep => {
                self.tick_envelopes();
                self.triangle.decrement_linear_counter();
                self.decrement_length_counters(clock);
                self.counter_suppression_cycles = 2;
            }
            FIFTH_STEP if clock.step_mode() == StepMode::FiveStep => {
                self.tick_envelopes();
                self.triangle.decrement_linear_counter();
                self.decrement_length_counters(clock);
                self.counter_suppression_cycles = 2;
            }
            _ => { /* Do nothing. */ }
        }
    }

    fn tick_envelopes(&mut self) {
        self.pulse_1.tick_envelope();
        self.pulse_2.tick_envelope();
        self.noise.tick_envelope();
    }

    fn decrement_length_counters(&mut self, clock: &ApuClock) {
        self.pulse_1.length_counter.decrement_towards_zero();
        self.pulse_2.length_counter.decrement_towards_zero();
        self.triangle.length_counter.decrement_towards_zero();
        self.noise.length_counter.decrement_towards_zero();

        self.pulse_1.tick_sweep();
        self.pulse_2.tick_sweep();

        info!(target: "apuevents", "Decremented length counters. P1: {}, P2: {}, T: {}, N: {}. APU Cycle: {}",
            self.pulse_1.length_counter, self.pulse_2.length_counter, self.triangle.length_counter,
            self.noise.length_counter, clock.cpu_cycle(),
        );
    }

    fn apply_length_counter_pending_values(&mut self) {
        self.pulse_1.length_counter.apply_pending_values();
        self.pulse_2.length_counter.apply_pending_values();
        self.triangle.length_counter.apply_pending_values();
        self.noise.length_counter.apply_pending_values();
    }

    fn maybe_set_frame_irq_pending(&mut self, clock: &ApuClock, cpu_pinout: &mut CpuPinout) {
        if clock.is_in_frame_irq_window() {
            if !self.suppress_frame_irq {
                cpu_pinout.assert_frame_irq();
            }

            self.frame_irq_status = if clock.cpu_cycle() == 0 {
                !self.suppress_frame_irq
            } else {
                true
            };
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Status {
    dmc_interrupt: bool,
    frame_irq_status: bool,
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
                self.frame_irq_status,
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

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum ClockResetStatus {
    Inactive,
    Pending,
    Ready,
}

impl ClockResetStatus {
    fn initialize(&mut self) {
        *self = Self::Pending;
    }

    fn tick(&mut self) -> bool {
        let is_ready = *self == Self::Ready;
        match self {
            Self::Inactive => { /* Stay inactive. */ }
            Self::Pending => *self = Self::Ready,
            Self::Ready => *self = Self::Inactive,
        }

        is_ready
    }
}
