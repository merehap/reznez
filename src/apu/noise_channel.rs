use crate::apu::length_counter::LengthCounter;
use crate::apu::timer::Timer;
use crate::util::integer::U4;

#[derive(Default)]
pub struct NoiseChannel {
    pub(super) enabled: bool,

    constant_volume: bool,
    volume_or_envelope: U4,

    should_loop: bool,
    period: U4,
    timer: Timer,
    pub(super) length_counter: LengthCounter,
}

impl NoiseChannel {
    pub fn write_control_byte(&mut self, value: u8) {
        self.length_counter.set_halt((value & 0b0010_0000) != 0);
        self.constant_volume =       (value & 0b0001_0000) != 0;
        self.volume_or_envelope =    (value & 0b0000_1111).into();
    }

    pub fn write_loop_and_period_byte(&mut self, value: u8) {
        self.should_loop = (value & 0b1000_0000) != 0;
        self.period =      (value & 0b0000_1111).into();
    }

    pub fn write_length_byte(&mut self, value: u8) {
        if self.enabled {
            self.length_counter.set_count_from_lookup((value & 0b1111_1000) >> 3);
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

    pub(super) fn step(&mut self) {
        let _wrapped_around = self.timer.tick();
    }
}
