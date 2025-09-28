use crate::counter::irq_counter_info::IrqCounterInfo;

pub struct IncrementingCounter {
    trigger_target: u16,
    ticking_enabled: bool,
    triggering_enabled: bool,
    count: u16,
}

impl IncrementingCounter {
    pub fn count_low_byte(&self) -> u8 {
        self.count.to_be_bytes()[1]
    }

    pub fn count_high_byte(&self) -> u8 {
        self.count.to_be_bytes()[0]
    }

    pub fn set_count_low_byte(&mut self, value: u8) {
        self.count = (self.count & 0xFF00) | u16::from(value);
    }

    pub fn set_count_high_byte(&mut self, value: u8) {
        self.count = (self.count & 0x00FF) | (u16::from(value) << 8);
    }

    pub fn tick(&mut self) -> bool {
        let old_count = self.count;
        if self.ticking_enabled {
            self.count = self.count.saturating_add(1);
        }

        let new_count = self.count;
        let trigger_if_enabled = new_count == self.trigger_target && old_count < new_count;
        let triggered = trigger_if_enabled && self.triggering_enabled;
        triggered
    }

    pub fn to_irq_counter_info(&self) -> IrqCounterInfo {
        IrqCounterInfo { ticking_enabled: self.ticking_enabled, triggering_enabled: self.triggering_enabled, count: self.count }
    }
}

#[derive(Clone, Copy)]
pub struct IncrementingCounterBuilder {
    trigger_target: Option<u16>,
}

impl IncrementingCounterBuilder {
    pub const fn new() -> Self {
        Self {
            trigger_target: None,
        }
    }

    pub const fn trigger_target(&mut self, trigger_target: u16) -> &mut Self {
        self.trigger_target = Some(trigger_target);
        self
    }

    pub const fn build(self) -> IncrementingCounter {
        IncrementingCounter {
            trigger_target: self.trigger_target.unwrap(),
            ticking_enabled: true,
            triggering_enabled: true,
            count: 0,
        }
    }
}