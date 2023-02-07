use crate::apu::timer::Timer;
use crate::apu::length_counter::LengthCounter;
use crate::util::integer::U7;

#[derive(Default)]
pub struct TriangleChannel {
    pub(super) enabled: bool,

    counter_control: bool,
    linear_counter_load: U7,
    timer: Timer,
    length_counter: LengthCounter,
}

impl TriangleChannel {
    pub fn write_control_byte(&mut self, value: u8) {
        self.counter_control =     (value & 0b1000_0000) != 0;
        self.linear_counter_load = (value & 0b0111_1111).into();
    }

    pub fn write_timer_low_byte(&mut self, value: u8) {
        self.timer.set_period_low(value);
    }

    pub fn write_length_and_timer_high_byte(&mut self, value: u8) {
        if self.enabled {
            self.length_counter = LengthCounter::from_lookup((value & 0b1111_1000) >> 3);
        }

        self.timer.set_period_high_and_reset_index(value & 0b0000_0111);
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

    pub(super) fn step(&mut self) -> bool {
        self.timer.tick()
    }
}
