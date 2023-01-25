use bitvec::prelude::BitVec;

use crate::util::integer::{U4, U7};

pub struct Dmc {
    enabled: bool,

    pub(super) irq_enabled: bool,
    should_loop: bool,
    frequency: U4,
    load_counter: U7,
    sample_address: u8,

    sample_buffer: BitVec,
    sample_bytes_remaining: SampleBytesRemaining,
}

impl Dmc {
    pub fn write_control_byte(&mut self, value: u8) {
        self.irq_enabled = (value & 0b1000_0000) != 0;
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
        self.sample_bytes_remaining.load_new_length_byte(value);
    }

    pub(super) fn enable_or_disable(&mut self, enable: bool) {
        self.enabled = enable;
        if !self.enabled {
            self.sample_bytes_remaining.clear();
            self.irq_enabled = false;
        }
    }

    pub(super) fn active(&self) -> bool {
        !self.sample_bytes_remaining.is_zero()
    }
}

impl Default for Dmc {
    fn default() -> Self {
        Dmc {
            enabled: Default::default(),
            irq_enabled: Default::default(),
            should_loop: Default::default(),
            frequency: Default::default(),
            load_counter: Default::default(),
            sample_address: Default::default(),
            sample_buffer: BitVec::with_capacity(8),
            sample_bytes_remaining: Default::default(),
        }
    }
}

#[derive(Clone, Copy, Default)]
struct SampleBytesRemaining(u16);

impl SampleBytesRemaining {
    fn load_new_length_byte(&mut self, length: u8) {
        self.0 = ((length as u16) << 4) | 1;
    }

    fn is_zero(self) -> bool {
        self.0 == 0
    }

    fn clear(&mut self) {
        self.0 = 0;
    }

    fn decrement(&mut self) {
        assert!(self.0 != 0);

        self.0 -= 1;
    }
}
