use crate::apu::timer::Timer;
use crate::util::integer::{U3, U4, U5};

#[derive(Default)]
pub struct PulseChannel {
    pub(super) enabled: bool,

    duty: Duty,
    length_counter_halt: bool,
    constant_volume: bool,
    volume_or_envelope: U4,

    sweep: Sweep,
    timer: Timer,
    length_counter: U5,
}

impl PulseChannel {
    pub fn write_control_byte(&mut self, value: u8) {
        self.duty =               ((value & 0b1100_0000) >> 6).into();
        self.length_counter_halt = (value & 0b0010_0000) != 0;
        self.constant_volume =     (value & 0b0001_0000) != 0;
        self.volume_or_envelope =  (value & 0b0000_1111).into();
    }

    pub fn write_sweep_byte(&mut self, value: u8) {
        self.sweep = Sweep::from_u8(value);
    }

    pub fn write_timer_low_byte(&mut self, value: u8) {
        self.timer.set_period_low(value);
    }

    pub fn write_length_and_timer_high_byte(&mut self, value: u8) {
        self.length_counter =                    ((value & 0b1110_0000) >> 5).into();
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

#[derive(Default)]
pub enum Duty {
    #[default]
    Low     = 0b0100_0000,
    Medium  = 0b0110_0000,
    High    = 0b0111_1000,
    Negated = 0b1001_1111,
}

impl From<u8> for Duty {
    fn from(value: u8) -> Self {
        match value {
            0 => Duty::Low,
            1 => Duty::Medium,
            2 => Duty::High,
            3 => Duty::Negated,
            _ => unreachable!(),
        }
    }
}

#[derive(Default)]
pub struct Sweep {
    enabled: bool,
    period: U3,
    period_change: PeriodChange,
    shift_count: U3,
}

impl Sweep {
    fn from_u8(value: u8) -> Sweep {
        Sweep {
            enabled:        (value & 0b1000_0000) != 0,
            period:        ((value & 0b0111_0000) >> 4).into(),
            period_change: ((value & 0b0000_1000) >> 3).into(),
            shift_count:    (value & 0b0000_0111).into(),
        }
    }
}

#[derive(Default)]
pub enum PeriodChange {
    #[default]
    Increase,
    Decrease,
}

impl From<u8> for PeriodChange {
    fn from(item: u8) -> Self {
        match item {
            0 => PeriodChange::Increase,
            1 => PeriodChange::Decrease,
            _ => unreachable!(),
        }
    }
}
