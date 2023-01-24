use crate::util::integer::{U5, U7, U11};

#[derive(Default)]
pub struct TriangleChannel {
    pub(super) enabled: bool,

    counter_control: bool,
    linear_counter_load: U7,
    timer: U11,
    length_counter_load: U5,
}

impl TriangleChannel {
    pub fn write_control_byte(&mut self, value: u8) {
        self.counter_control =     (value & 0b1000_0000) != 0;
        self.linear_counter_load = (value & 0b0111_1111).into();
    }

    pub fn write_timer_low_byte(&mut self, value: u8) {
        self.timer.set_low_byte(value);
    }

    pub fn write_lcl_and_timer_high_byte(&mut self, value: u8) {
        self.length_counter_load = ((value & 0b1111_1000) >> 3).into();
        self.timer.set_high_bits(    value & 0b0000_0111);
    }
}
