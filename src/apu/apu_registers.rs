#![expect(clippy::many_single_char_names)]

use log::info;
use splitbits::{combinebits, splitbits};

use crate::apu::apu_clock::{ApuClock, StepMode};
use crate::apu::sweep::NegateBehavior;
use crate::apu::pulse_channel::PulseChannel;
use crate::apu::triangle_channel::TriangleChannel;
use crate::apu::noise_channel::NoiseChannel;
use crate::apu::dmc::Dmc;
use crate::bus::Bus;
use crate::cpu::dmc_dma::DmcDma;
use crate::memory::read_result::ReadResult;
use crate::memory::cpu::cpu_pinout::CpuPinout;
use crate::util::circular_buffer::CircularBuffer;

const STORED_SAMPLE_COUNT: u32 = 5 * 1000;

pub struct ApuRegisters {
    pub pulse_1: PulseChannel<{NegateBehavior::OnesComplement}>,
    pub pulse_2: PulseChannel<{NegateBehavior::TwosComplement}>,
    pub triangle: TriangleChannel,
    pub noise: NoiseChannel,
    pub dmc: Dmc,

    pub mixed_samples: Box<[f32; 40]>,

    pub pulse1_volumes: CircularBuffer<f64>,
    pub pulse2_volumes: CircularBuffer<f64>,
    pub triangle_volumes: CircularBuffer<f64>,
    pub noise_volumes: CircularBuffer<f64>,
    pub dmc_volumes: CircularBuffer<f64>,
    pub mixed_values: CircularBuffer<f64>,

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
    pub fn new() -> Self {
        Self {
            pulse_1: PulseChannel::default(),
            pulse_2: PulseChannel::default(),
            triangle: TriangleChannel::default(),
            noise: NoiseChannel::default(),
            dmc: Dmc::default(),

            mixed_samples: vec![0.0; 40].into_boxed_slice().try_into().unwrap(),

            pulse1_volumes: CircularBuffer::default_filled(STORED_SAMPLE_COUNT),
            pulse2_volumes: CircularBuffer::default_filled(STORED_SAMPLE_COUNT),
            triangle_volumes: CircularBuffer::default_filled(STORED_SAMPLE_COUNT),
            noise_volumes: CircularBuffer::default_filled(STORED_SAMPLE_COUNT),
            dmc_volumes: CircularBuffer::default_filled(STORED_SAMPLE_COUNT),
            mixed_values: CircularBuffer::default_filled(STORED_SAMPLE_COUNT),

            pending_step_mode: StepMode::FourStep,
            dmc_enabled: false,
            frame_irq_status: false,
            suppress_frame_irq: false,
            should_acknowledge_frame_irq: false,
            clock_reset_status: ClockResetStatus::Inactive,

            counter_suppression_cycles: 0,
        }
    }

    pub fn reset(bus: &mut Bus) {
        // At reset, $4015 should be cleared
        bus.apu_regs.disable_channels();
        CpuPinout::acknowledge_dmc_irq(bus);
        // At reset, $4017 should should be rewritten with last value written
        bus.apu_regs.clock_reset_status.initialize();
        info!(target: "apuevents", "Frame IRQ acknowledged by RESET. APU Cycle: {}", bus.apu_clock().cpu_cycle());
        CpuPinout::acknowledge_frame_irq(bus);
        bus.apu_regs.frame_irq_status = false;
    }

    pub fn clock_reset_status(&self) -> ClockResetStatus {
        self.clock_reset_status
    }

    // Peek 0x4015
    pub fn peek_status(&self, cpu_pinout: &CpuPinout, dma: &DmcDma) -> ReadResult {
        let i = cpu_pinout.dmc_irq_asserted();
        let f = self.frame_irq_status;
        let d = self.dmc_enabled && dma.enabled();
        let n = self.noise.active();
        let t = self.triangle.active();
        let q = self.pulse_2.active();
        let p = self.pulse_1.active();
        ReadResult::partial(combinebits!("if0d ntqp"), 0b1101_1111)
    }

    // Read 0x4015
    pub fn read_status(&mut self, clock: &ApuClock, cpu_pinout: &CpuPinout, dma: &DmcDma) -> ReadResult {
        if cpu_pinout.frame_irq_asserted() {
            info!(target: "apuevents", "Frame IRQ flag will be cleared during the next GET cycle due to APUStatus read. APU Cycle: {}", clock.cpu_cycle());
        }

        let status = self.peek_status(cpu_pinout, dma);
        // Clearing Frame IRQ pending must be delayed until the next GET cycle.
        self.should_acknowledge_frame_irq = true;
        status
    }

    // Write 0x4015
    pub fn write_status(bus: &mut Bus) {
        let value = bus.cpu_pinout.data_bus;
        info!(target: "apuevents", "APU status write: {value:05b} . APU Cycle: {}", bus.apu_clock().cpu_cycle());

        let enabled_channels = splitbits!(value, "...dntqp");
        Dmc::set_enabled(bus, enabled_channels.d);

        let apu_regs = &mut bus.apu_regs;
        // This applies immediately, unlike the similar flag within DMC.
        apu_regs.dmc_enabled = enabled_channels.d;

        apu_regs.noise.set_enabled(enabled_channels.n);
        apu_regs.triangle.set_enabled(enabled_channels.t);
        apu_regs.pulse_2.set_enabled(enabled_channels.q);
        apu_regs.pulse_1.set_enabled(enabled_channels.p);
    }

    // Upon RESET
    pub fn disable_channels(&mut self) {
        self.noise.set_enabled(false);
        self.triangle.set_enabled(false);
        self.pulse_2.set_enabled(false);
        self.pulse_1.set_enabled(false);
    }

    // Write 0x4017
    pub fn write_frame_counter(bus: &mut Bus) {
        let value = bus.cpu_pinout.data_bus;
        use StepMode::*;
        bus.apu_regs.pending_step_mode = if value & 0b1000_0000 == 0 { FourStep } else { FiveStep };
        bus.apu_regs.suppress_frame_irq = value & 0b0100_0000 != 0;
        if bus.apu_regs.suppress_frame_irq {
            info!(target: "apuevents", "Frame IRQ acknowledged by Frame Counter write. APU Cycle: {}", bus.apu_clock().cpu_cycle());
            CpuPinout::acknowledge_frame_irq(bus);
            bus.apu_regs.frame_irq_status = false;
        }

        bus.apu_regs.clock_reset_status.initialize();

        info!(target: "apuevents", "Frame counter write: {:?}, Suppress IRQ: {}, Status: {:?}, APU Cycle: {}",
            bus.apu_regs.pending_step_mode, bus.apu_regs.suppress_frame_irq, bus.apu_regs.clock_reset_status, bus.apu_clock().cpu_cycle());
    }

    pub fn tick_get(bus: &mut Bus) {
        if bus.apu_regs.counter_suppression_cycles > 0 {
            bus.apu_regs.counter_suppression_cycles -= 1;
        }

        bus.apu_regs.maybe_update_step_mode(&mut bus.master_clock.apu_clock);

        if bus.apu_regs.should_acknowledge_frame_irq {
            info!(target: "apuevents", "Frame IRQ acknowledged by APUSTATUS read. APU Cycle: {}", bus.apu_clock().cpu_cycle());
            CpuPinout::acknowledge_frame_irq(bus);
            bus.apu_regs.frame_irq_status = false;
            bus.apu_regs.should_acknowledge_frame_irq = false;
        }

        ApuRegisters::maybe_set_frame_irq_pending(bus);
        bus.apu_regs.apply_length_counter_pending_values();

        bus.apu_regs.triangle.tick();
        bus.apu_regs.dmc.tick(&mut bus.dmc_dma);
    }

    pub fn tick_put(bus: &mut Bus) {
        if bus.apu_regs.counter_suppression_cycles > 0 {
            bus.apu_regs.counter_suppression_cycles -= 1;
        }

        ApuRegisters::maybe_set_frame_irq_pending(bus);

        let clock = bus.master_clock.apu_clock();
        let apu_regs = &mut bus.apu_regs;
        let (quarter_frame, half_frame) = match (clock.apu_cycle(), clock.step_mode()) {
            (3728 , _                 ) => (true, false), // First quarter
            (7456 , _                 ) => (true, true),  // Second quarter (first half)
            (11185, _                 ) => (true, false), // Third quarter
            (14914, StepMode::FourStep) => (true, true),  // Final quarter (final half), 4-step
            (18640, StepMode::FiveStep) => (true, true),  // Final quarter (final half), 5-step
            _ => (false, false),
        };

        if quarter_frame {
            apu_regs.tick_envelopes();
            apu_regs.triangle.decrement_linear_counter();
            apu_regs.counter_suppression_cycles = 2;

            if half_frame {
                apu_regs.decrement_length_counters(clock);
                apu_regs.tick_sweeps();
            }
        }

        apu_regs.apply_length_counter_pending_values();

        apu_regs.pulse_1.tick_put();
        apu_regs.pulse_2.tick_put();
        apu_regs.noise.tick_put();
        apu_regs.triangle.tick();
        apu_regs.dmc.tick(&mut bus.dmc_dma);
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

        info!(target: "apuevents", "Decremented length counters. P1: {}, P2: {}, T: {}, N: {}. APU Cycle: {}",
            self.pulse_1.length_counter, self.pulse_2.length_counter, self.triangle.length_counter,
            self.noise.length_counter, clock.cpu_cycle(),
        );
    }

    fn tick_sweeps(&mut self) {
        self.pulse_1.tick_sweep();
        self.pulse_2.tick_sweep();
    }

    fn apply_length_counter_pending_values(&mut self) {
        self.pulse_1.length_counter.apply_pending_values();
        self.pulse_2.length_counter.apply_pending_values();
        self.triangle.length_counter.apply_pending_values();
        self.noise.length_counter.apply_pending_values();
    }

    fn maybe_set_frame_irq_pending(bus: &mut Bus) {
        if bus.apu_clock().is_in_frame_irq_window() {
            if !bus.apu_regs.suppress_frame_irq {
                CpuPinout::assert_frame_irq(bus);
            }

            bus.apu_regs.frame_irq_status = if bus.apu_clock().cpu_cycle() == 0 {
                !bus.apu_regs.suppress_frame_irq
            } else {
                true
            };
        }
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