use ux::u3;

#[derive(Default)]
pub struct FrequencyTimer {
    period: u16,
    index: u16,
}

impl FrequencyTimer {
    pub fn period(&self) -> u16 {
        self.period
    }

    pub fn set_period_and_reset_index(&mut self, period: u16) {
        self.period = period;
        self.index = self.period;
    }

    pub fn set_period_high_and_reset_index(&mut self, period_high: u3) {
        self.period &= 0b0000_0000_1111_1111;
        self.period |= u16::from(period_high) << 8;
        self.index = self.period;
    }

    pub fn set_period_low(&mut self, period_low: u8) {
        self.period &= 0b0000_0111_0000_0000;
        self.period |= u16::from(period_low);
    }

    // Only used by the Sweep unit of the Pulse channel.
    // TODO: See if this and set_period_and_reset_index can be consolidated.
    pub fn set_period(&mut self, period: u16) {
        self.period = period;
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
