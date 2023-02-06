use crate::util::integer::{U4, U5};

#[derive(Default)]
pub struct NoiseChannel {
    pub(super) enabled: bool,

    length_counter_halt: bool,
    constant_volume: bool,
    volume_or_envelope: U4,

    should_loop: bool,
    period: U4,
    length_counter: U5,
}

impl NoiseChannel {
    pub fn write_control_byte(&mut self, value: u8) {
        self.length_counter_halt = (value & 0b0010_0000) != 0;
        self.constant_volume =     (value & 0b0001_0000) != 0;
        self.volume_or_envelope =  (value & 0b0000_1111).into();
    }

    pub fn write_loop_and_period_byte(&mut self, value: u8) {
        self.should_loop = (value & 0b1000_0000) != 0;
        self.period =      (value & 0b0000_1111).into();
    }

    pub fn write_length_byte(&mut self, value: u8) {
        if self.enabled {
            self.length_counter = ((value & 0b1111_1000) >> 3).into();
        }
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
}
