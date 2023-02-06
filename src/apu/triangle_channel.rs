use crate::apu::timer::Timer;
use crate::util::integer::{U5, U7};

#[derive(Default)]
pub struct TriangleChannel {
    pub(super) enabled: bool,

    counter_control: bool,
    linear_counter_load: U7,
    timer: Timer,
    length_counter: U5,
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
            self.length_counter = ((value & 0b1111_1000) >> 3).into();
        }

        self.timer.set_period_high_and_reset_index(value & 0b0000_0111);
    }

    pub(super) fn enable_or_disable(&mut self, enable: bool) {
        self.enabled = enable;
        if !self.enabled {
            self.length_counter = U5::ZERO;
        }
    }

    pub(super) fn active(&self) -> bool {
        self.length_counter != U5::ZERO
    }

    pub(super) fn step(&mut self) -> bool {
        self.timer.tick()
    }
}
