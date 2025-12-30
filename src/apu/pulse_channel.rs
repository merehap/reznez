use splitbits::splitbits_ux;
use ux::{u2, u4};

use crate::apu::apu_registers::CycleParity;
use crate::apu::length_counter::LengthCounter;
use crate::apu::frequency_timer::FrequencyTimer;
use crate::util::bit_util;

//                  Sweep -----> Timer
//                    |            |
//                    |            |
//                    |            v
//                    |        Sequencer   Length Counter
//                    |            |             |
//                    |            |             |
//                    v            v             v
// Envelope -------> Gate -----> Gate -------> Gate ---> (to mixer)
#[derive(Default)]
pub struct PulseChannel {
    pub(super) length_counter: LengthCounter,

    enabled: bool,

    constant_volume: bool,
    volume_or_envelope: u4,

    frequency_timer: FrequencyTimer,
    sequencer: Sequencer,
}

impl PulseChannel {
    // Write $4000 or $4004
    pub fn write_control_byte(&mut self, value: u8) {
        let fields = splitbits_ux!(value, "ddhc eeee");
        self.sequencer.set_duty(fields.d.into());
        self.length_counter.start_halt(fields.h);
        self.constant_volume = fields.c;
        self.volume_or_envelope = fields.e;
    }

    // Write $4001 or $4005
    #[allow(clippy::unused_self)]
    pub fn write_sweep_byte(&mut self, _value: u8) {

    }

    // Write $4002 or $4006
    pub fn write_timer_low_byte(&mut self, value: u8) {
        self.frequency_timer.set_period_low(value);
    }

    // Write $4003 or $4007
    pub fn write_length_and_timer_high_byte(&mut self, value: u8) {
        if self.enabled {
            self.length_counter.start_reload((value & 0b1111_1000) >> 3);
        }

        self.sequencer.reset();
        self.frequency_timer.set_period_high_and_reset_index(value & 0b0000_0111);
    }

    // Write 0x4015
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
            let triggered = self.frequency_timer.tick();
            if triggered {
                self.sequencer.step();
            }
        }
    }

    pub(super) fn sample_volume(&self) -> u8 {
        let on_duty = self.sequencer.on_duty();
        let non_short_period = self.frequency_timer.period() >= 8;
        let non_zero_length = !self.length_counter.is_zero();

        let enabled = self.enabled && on_duty && non_short_period && non_zero_length;
        if enabled {
            self.volume_or_envelope.into()
        } else {
            0
        }
    }
}

#[derive(Default)]
pub struct Sequencer {
    index: u32,
    duty: Duty,
}

impl Sequencer {
    pub fn reset(&mut self) {
        self.index = 0;
    }

    pub fn step(&mut self) {
        self.index += 1;
        self.index %= 8;
    }

    pub fn on_duty(&self) -> bool {
        bit_util::get_bit(self.duty as u8, self.index)
    }

    pub fn set_duty(&mut self, duty: Duty) {
        self.duty = duty;
    }
}

#[derive(Clone, Copy, Default)]
pub enum Duty {
    #[default]
    Low     = 0b0100_0000,
    Medium  = 0b0110_0000,
    High    = 0b0111_1000,
    Negated = 0b1001_1111,
}

impl From<u2> for Duty {
    fn from(value: u2) -> Self {
        match u8::from(value) {
            0 => Duty::Low,
            1 => Duty::Medium,
            2 => Duty::High,
            3 => Duty::Negated,
            _ => unreachable!(),
        }
    }
}