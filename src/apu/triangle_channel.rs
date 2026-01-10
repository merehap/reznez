use splitbits::splitbits_named_ux;

use ux::u7;

use crate::apu::frequency_timer::FrequencyTimer;
use crate::apu::length_counter::LengthCounter;

const VOLUME_SEQUENCE: [u8; 0x20] = [
    15, 14, 13, 12, 11, 10,  9,  8,  7,  6,  5,  4,  3,  2,  1,  0,
     0,  1,  2,  3,  4,  5,  6,  7,  8,  9, 10, 11, 12, 13, 14, 15,
];

#[derive(Default)]
pub struct TriangleChannel {
    pub(super) enabled: bool,

    counter_control: bool,
    linear_counter_reload_value: u7,
    linear_counter: u7,
    frequency_timer: FrequencyTimer,
    pub(super) length_counter: LengthCounter,

    linear_counter_reload: bool,

    sequence_index: usize,
}

impl TriangleChannel {
    // Write 0x4008
    pub fn set_control_and_linear(&mut self, value: u8) {
        (self.counter_control, self.linear_counter) = splitbits_named_ux!(value, "clll llll");
        self.length_counter.start_halt(self.counter_control);
    }

    // Write 0x400A
    pub fn set_timer_low(&mut self, value: u8) {
        self.frequency_timer.set_period_low(value);
    }

    // Write 0x400B
    pub fn set_length_and_timer_high(&mut self, value: u8) {
        let (length, period) = splitbits_named_ux!(value, "llll lppp");
        if self.enabled {
            self.length_counter.start_reload(length);
        }

        self.frequency_timer.set_period_high_and_reset_index(period);
        self.linear_counter_reload = true;
    }

    // Read 0x4015
    pub(super) fn active(&self) -> bool {
        !self.length_counter.is_zero()
    }

    // Write 0x4015
    pub(super) fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !self.enabled {
            self.length_counter.set_to_zero();
        }
    }

    // Every CPU cycle.
    pub(super) fn tick(&mut self) {
        let triggered = self.frequency_timer.tick();
        if triggered && !self.length_counter.is_zero() && self.linear_counter > u7::new(0) {
            self.sequence_index += 1;
            self.sequence_index %= 0x20;
        }
    }

    // Every quarter frame.
    pub(super) fn decrement_linear_counter(&mut self) {
        if self.linear_counter_reload {
            self.linear_counter = self.linear_counter_reload_value;
        } else if self.linear_counter > u7::new(0) {
            self.linear_counter = self.linear_counter - u7::new(1);
        }

        if !self.counter_control {
            self.linear_counter_reload = false;
        }
    }

    pub(super) fn sample_volume(&self) -> u8 {
        if !self.length_counter.is_zero() && self.linear_counter > u7::new(0) {
            VOLUME_SEQUENCE[self.sequence_index]
        } else {
            0
        }
    }
}
