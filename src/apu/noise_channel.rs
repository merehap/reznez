use splitbits::{splitbits, splitbits_ux};
use ux::{u4, u15};

use crate::apu::apu_registers::CycleParity;
use crate::apu::length_counter::LengthCounter;
use crate::apu::timer::Timer;

const NTSC_PERIODS: [u16; 16] =
    [4, 8, 16, 32, 64, 96, 128, 160, 202, 254, 380, 508, 762, 1016, 2034, 4068];

// TODO: splitbits
pub struct NoiseChannel {
    pub(super) enabled: bool,

    constant_volume: bool,
    volume_or_envelope: u4,

    mode: bool,
    timer: Timer,
    pub(super) length_counter: LengthCounter,

    shift_register: LinearFeedbackShiftRegister,
}

impl Default for NoiseChannel {
    fn default() -> NoiseChannel {
        NoiseChannel {
            enabled: false,

            constant_volume: false,
            volume_or_envelope: u4::new(0),

            mode: false,
            timer: Timer::default(),
            length_counter: LengthCounter::default(),

            shift_register: LinearFeedbackShiftRegister::new(),
        }
    }
}

impl NoiseChannel {
    // Write 0x400C
    pub fn set_control(&mut self, value: u8) {
        let fields = splitbits_ux!(value, "..hc eeee");
        self.length_counter.start_halt(fields.h);
        self.constant_volume = fields.c;
        self.volume_or_envelope = fields.e;
    }

    // Write 0x400E
    pub fn set_loop_and_period(&mut self, value: u8) {
        let fields = splitbits!(value, "m... pppp");
        self.mode = fields.m;
        let period = NTSC_PERIODS[fields.p as usize];
        self.timer.set_period_and_reset_index(period);
    }

    // Write 0x400F
    pub fn set_length(&mut self, value: u8) {
        if self.enabled {
            self.length_counter.start_reload(value >> 3);
        }
    }

    pub(super) fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !self.enabled {
            self.length_counter.set_to_zero();
        }
    }

    pub(super) fn active(&self) -> bool {
        !self.length_counter.is_zero()
    }

    pub(super) fn tick(&mut self, parity: CycleParity) {
        if parity == CycleParity::Put {
            let triggered = self.timer.tick();
            if triggered {
                let feedback_index = if self.mode { 6 } else { 1 };
                self.shift_register.step(feedback_index);
            }
        }
    }

    pub(super) fn sample_volume(&self) -> u8 {
        if self.length_counter.is_zero() || !self.shift_register.low_bit() {
            0
        } else {
            u8::from(self.volume_or_envelope)
        }
    }
}

pub struct LinearFeedbackShiftRegister(u15);

impl LinearFeedbackShiftRegister {
    pub fn new() -> Self {
        Self(u15::new(0b000_0000_0000_0001))
    }

    pub fn step(&mut self, feedback_index: u8) {
        let feedback = self.bit(0) ^ self.bit(feedback_index);
        self.0 >>= 1;
        self.0 |= u15::new(feedback as u16) << 14;
    }

    pub fn low_bit(&self) -> bool {
        self.bit(0)
    }

    fn bit(&self, index: u8) -> bool {
        u16::from(self.0 >> index) & 1 == 1
    }
}