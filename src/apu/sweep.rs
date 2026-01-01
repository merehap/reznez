use std::marker::ConstParamTy;

use ux::{u3, u11};

use crate::apu::frequency_timer::FrequencyTimer;

#[derive(Default)]
pub struct Sweep<const N: NegateBehavior> {
    enabled: bool,
    divider: Divider,
    negate: bool,
    shift_count: u3,
    target_period: Option<u11>,
    frequency_timer: FrequencyTimer,
}

impl <const N: NegateBehavior> Sweep<N> {
    // Write $4001 or $4005
    pub fn set(&mut self, enabled: bool, period: u3, negate: bool, shift_count: u3) {
        self.enabled = enabled;
        self.negate = negate;
        self.shift_count = shift_count;

        self.divider.set_period(period);
        self.divider.prepare_to_reload();
    }

    // Write $4002 or $4006
    pub fn set_current_period_low(&mut self, value: u8) {
        self.frequency_timer.set_period_low(value);
        self.target_period = Some(self.frequency_timer.period().try_into().unwrap());
    }

    // Write $4003 or $4007
    pub fn set_current_period_high_and_reset_index(&mut self, value: u3) {
        self.frequency_timer.set_period_high_and_reset_index(value);
        self.target_period = Some(self.frequency_timer.period().try_into().unwrap());
    }

    // Every cycle
    pub fn update_target_period(&mut self) {
        let current_period = self.current_period();
        let change_amount = current_period >> u8::from(self.shift_count);
        self.target_period = if self.negate {
            Some(current_period.checked_sub(N.magnitude(change_amount)).unwrap_or(u11::ZERO))
        } else {
            current_period.checked_add(change_amount)
        };
    }

    // Every PUT cycle
    pub fn tick_frequency_timer(&mut self) -> bool {
        self.frequency_timer.tick()
    }

    // Every APU half-frame cycle
    pub fn tick(&mut self) {
        let divider_triggered = self.divider.tick();
        if let Some(target_period) = self.target_period
                && self.enabled
                && divider_triggered
                && self.shift_count > u3::new(0) {
            self.frequency_timer.set_period(u16::from(target_period));
        }
    }

    pub fn muting(&self) -> bool {
        let short_period = self.current_period() < u11::new(8);
        let sweep_target_overflowed = self.target_period.is_none();
        short_period || sweep_target_overflowed
    }

    fn current_period(&self) -> u11 {
        self.frequency_timer.period().try_into().unwrap()
    }
}

#[derive(Default)]
pub struct Divider {
    period: u3,
    index: u3,
    should_reload: bool,
}

impl Divider {
    pub fn set_period(&mut self, period: u3) {
        self.period = period;
        self.should_reload = true;
    }

    pub fn prepare_to_reload(&mut self) {
        self.should_reload = true;
    }

    pub fn reload(&mut self) {
        self.index = self.period;
    }

    pub fn tick(&mut self) -> bool {
        self.index = self.index - u3::new(1);

        let triggered = self.index == u3::new(0);
        if triggered || self.should_reload {
            self.should_reload = false;
            self.reload();
        }

        triggered
    }
}

#[derive(PartialEq, Eq, Clone, Copy, ConstParamTy)]
pub enum NegateBehavior {
    OnesComplement,
    TwosComplement,
}

impl NegateBehavior {
    pub fn magnitude(self, value: u11) -> u11 {
        match self {
            Self::OnesComplement => value + u11::ONE,
            Self::TwosComplement => value,
        }
    }
}

trait U11Ext {
    const ZERO: u11 = u11::new(0);
    const ONE: u11 = u11::new(1);

    fn checked_add(self, other: u11) -> Option<u11>;
    fn checked_sub(self, other: u11) -> Option<u11>;
}

impl U11Ext for u11 {
    fn checked_add(self, other: u11) -> Option<u11> {
        (u16::from(self) + u16::from(other)).try_into().ok()
    }

    fn checked_sub(self, other: u11) -> Option<u11> {
        u16::from(self).checked_sub(u16::from(other))
            .and_then(|result| result.try_into().ok())
    }
}