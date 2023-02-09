use crate::apu::timer::Timer;
use crate::apu::length_counter::LengthCounter;
use crate::util::integer::U7;

const VOLUME_SEQUENCE: [u8; 0x20] = [
    15, 14, 13, 12, 11, 10,  9,  8,  7,  6,  5,  4,  3,  2,  1,  0,
     0,  1,  2,  3,  4,  5,  6,  7,  8,  9, 10, 11, 12, 13, 14, 15,
];

#[derive(Default)]
pub struct TriangleChannel {
    pub(super) enabled: bool,

    counter_control: bool,
    linear_counter_reload_value: U7,
    linear_counter: U7,
    timer: Timer,
    pub(super) length_counter: LengthCounter,

    linear_counter_reload: bool,

    sequence_index: usize,
}

impl TriangleChannel {
    pub fn write_control_byte(&mut self, value: u8) {
        self.counter_control =     (value & 0b1000_0000) != 0;
        self.linear_counter = (value & 0b0111_1111).into();

        self.length_counter.set_halt(self.counter_control);
    }

    pub fn write_timer_low_byte(&mut self, value: u8) {
        self.timer.set_period_low(value);
    }

    pub fn write_length_and_timer_high_byte(&mut self, value: u8) {
        if self.enabled {
            self.length_counter.set_count_from_lookup((value & 0b1111_1000) >> 3);
        }

        self.timer.set_period_high_and_reset_index(value & 0b0000_0111);

        self.linear_counter_reload = true;
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

    pub(super) fn step_quarter_frame(&mut self) {
        self.advance_timer_and_sequence_index();
        if self.linear_counter_reload {
            self.linear_counter = self.linear_counter_reload_value;
        } else {
            self.linear_counter.decrement_towards_zero();
        }

        if !self.counter_control {
            self.linear_counter_reload = false;
        }
    }

    pub(super) fn step_half_frame(&mut self) {
        self.advance_timer_and_sequence_index();
    }

    pub(super) fn sample_volume(&self) -> f32 {
        if !self.length_counter.is_zero() && self.linear_counter != U7::ZERO {
            f32::from(VOLUME_SEQUENCE[self.sequence_index])
        } else {
            0.0
        }
    }

    fn advance_timer_and_sequence_index(&mut self) {
        let wrapped_around = self.timer.tick();
        if wrapped_around && !self.length_counter.is_zero() && self.linear_counter != U7::ZERO {
            self.sequence_index += 1;
            self.sequence_index %= 0x20;
        }
    }
}
