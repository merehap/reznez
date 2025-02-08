use crate::apu::length_counter::LengthCounter;
use crate::apu::timer::Timer;
use crate::util::integer::U4;

const NTSC_PERIODS: [u16; 16] =
    [4, 8, 16, 32, 64, 96, 128, 160, 202, 254, 380, 508, 762, 1016, 2034, 4068];

// TODO: splitbits
pub struct NoiseChannel {
    pub(super) enabled: bool,

    constant_volume: bool,
    volume_or_envelope: U4,

    mode: bool,
    timer: Timer,
    pub(super) length_counter: LengthCounter,

    shift_register: u16,
}

impl Default for NoiseChannel {
    fn default() -> NoiseChannel {
        NoiseChannel {
            enabled: false,

            constant_volume: false,
            volume_or_envelope: U4::default(),

            mode: false,
            timer: Timer::default(),
            length_counter: LengthCounter::default(),

            shift_register: 0b000_0000_0000_0001,
        }
    }
}

impl NoiseChannel {
    pub fn write_control_byte(&mut self, value: u8) {
        self.length_counter.start_halt((value & 0b0010_0000) != 0);
        self.constant_volume =         (value & 0b0001_0000) != 0;
        self.volume_or_envelope =      (value & 0b0000_1111).into();
    }

    pub fn write_loop_and_period_byte(&mut self, value: u8) {
        self.mode = (value & 0b1000_0000) != 0;
        let period = NTSC_PERIODS[(value & 0b0000_1111) as usize];
        self.timer.set_period_and_reset_index(period);
    }

    // Write 0x400F
    pub fn write_length_byte(&mut self, value: u8) {
        if self.enabled {
            self.length_counter.start_reload((value & 0b1111_1000) >> 3);
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

    pub(super) fn execute_put_cycle(&mut self) {
        let wrapped_around = self.timer.tick();
        if wrapped_around {
            let mut feedback = self.shift_register & 1;
            if self.mode {
                feedback ^= (self.shift_register & 0b100_0000) >> 6;
            } else {
                feedback ^= (self.shift_register & 0b000_0010) >> 1;
            };

            feedback <<= 14;

            self.shift_register >>= 1;
            self.shift_register |= feedback;
        }
    }

    pub(super) fn sample_volume(&self) -> f32 {
        if self.length_counter.is_zero() || self.shift_register & 1 == 0 {
            0.0
        } else {
            f32::from(self.volume_or_envelope.to_u8())
        }
    }
}
