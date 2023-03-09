#[derive(Default)]
pub struct Timer {
    period: u16,
    index: u16,
}

impl Timer {
    pub fn period(&self) -> u16 {
        self.period
    }

    pub fn set_period_and_reset_index(&mut self, period: u16) {
        self.period = period;
        self.index = self.period;
    }

    pub fn set_period_high_and_reset_index(&mut self, period_high: u8) {
        self.period &= 0b0000_0000_1111_1111;
        self.period |= u16::from(period_high & 0b0000_0111) << 8;
        self.index = self.period;
    }

    pub fn set_period_low(&mut self, period_low: u8) {
        self.period &= 0b0000_0111_0000_0000;
        self.period |= u16::from(period_low);
    }

    pub fn tick(&mut self) -> bool {
        let mut wrapped_around = false;
        match (self.period, self.index) {
            (0, _) => self.index = 0,
            (_, 0) => {
                self.index = self.period;
                wrapped_around = true;
            }
            (_, _) => self.index -= 1,
        }

        wrapped_around
    }
}
