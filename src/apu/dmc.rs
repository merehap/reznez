use crate::util::integer::{U4, U7};

#[derive(Default)]
pub struct Dmc {
    pub(super) enabled: bool,

    irq_enable: bool,
    should_loop: bool,
    frequency: U4,
    load_counter: U7,
    sample_address: u8,
    sample_length: u8,
}

impl Dmc {
    pub fn write_control_byte(&mut self, value: u8) {
        self.irq_enable =  (value & 0b1000_0000) != 0;
        self.should_loop = (value & 0b0100_0000) != 0;
        self.frequency =   (value & 0b0000_1111).into();
    }

    pub fn write_load_counter(&mut self, value: u8) {
        self.load_counter = (value & 0b0111_1111).into();
    }

    pub fn write_sample_address(&mut self, value: u8) {
        self.sample_address = value;
    }

    pub fn write_sample_length(&mut self, value: u8) {
        self.sample_length = value;
    }
}
