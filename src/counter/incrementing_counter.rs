use crate::counter::irq_counter_info::IrqCounterInfo;
use crate::mapper::WhenDisabledPrevent;

pub struct IncrementingCounter {
    auto_triggered_by: IncAutoTriggeredBy,
    trigger_target: u16,
    when_target_reached: WhenTargetReached,
    when_disabled_prevent: Option<WhenDisabledPrevent>,

    // State
    ticking_enabled: bool,
    triggering_enabled: bool,
    count: u16,
}

impl IncrementingCounter {
    pub fn enable(&mut self) {
        assert!(self.when_disabled_prevent.is_some(), "This counter is configured to never be disabled, so it starts enabled.");
        self.triggering_enabled = true;
        self.ticking_enabled = true;
    }

    pub fn disable(&mut self) {
        match self.when_disabled_prevent {
            // TODO: Make a wrapper type that doesn't allow enabling/disabling instead of failing at runtime.
            None => panic!("Can't disable since this counter is configured to never be disabled."),
            Some(WhenDisabledPrevent::Ticking) => self.ticking_enabled = false,
            Some(WhenDisabledPrevent::Triggering) => self.triggering_enabled = false,
            Some(WhenDisabledPrevent::TickingAndTriggering) => {
                self.ticking_enabled = false;
                self.triggering_enabled = false;
            }
        }
    }

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

    pub fn clear(&mut self) {
        self.count = 0;
    }

    pub fn tick(&mut self) -> bool {
        let old_count = self.count;
        if self.ticking_enabled {
            let target_reached = self.count == self.trigger_target;
            match (target_reached, self.when_target_reached) {
                (true, WhenTargetReached::Stay) => { /* Stay on the old count. */ }
                (true, WhenTargetReached::Clear) => self.count = 0,
                (false, _) | (_, WhenTargetReached::Continue) => self.count = self.count.wrapping_add(1),
            }
        }

        let new_count = self.count;
        let trigger_if_enabled = match self.auto_triggered_by {
            IncAutoTriggeredBy::AlreadyOnTarget => old_count == self.trigger_target,
            IncAutoTriggeredBy::EndingOnTarget => new_count == self.trigger_target && old_count != new_count,
        };
        let triggered = trigger_if_enabled && self.triggering_enabled;
        triggered
    }

    pub fn to_irq_counter_info(&self) -> IrqCounterInfo {
        IrqCounterInfo { ticking_enabled: self.ticking_enabled, triggering_enabled: self.triggering_enabled, count: self.count }
    }
}

#[derive(Clone, Copy)]
pub struct IncrementingCounterBuilder {
    auto_triggered_by: Option<IncAutoTriggeredBy>,
    trigger_target: Option<u16>,
    when_target_reached: Option<WhenTargetReached>,
    when_disabled_prevent: Option<Option<WhenDisabledPrevent>>,
}

impl IncrementingCounterBuilder {
    pub const fn new() -> Self {
        Self {
            auto_triggered_by: None,
            trigger_target: None,
            when_target_reached: None,
            when_disabled_prevent: None,
        }
    }

    pub const fn auto_triggered_by(&mut self, auto_triggered_by: IncAutoTriggeredBy) -> &mut Self {
        self.auto_triggered_by = Some(auto_triggered_by);
        self
    }

    pub const fn trigger_target(&mut self, trigger_target: u16) -> &mut Self {
        self.trigger_target = Some(trigger_target);
        self
    }

    pub const fn when_target_reached(&mut self, when_target_reached: WhenTargetReached) -> &mut Self {
        self.when_target_reached = Some(when_target_reached);
        self
    }

    pub const fn when_disabled_prevent(&mut self, when_disabled_prevent: WhenDisabledPrevent) -> &mut Self {
        self.when_disabled_prevent = Some(Some(when_disabled_prevent));
        self
    }

    pub const fn never_disabled(&mut self) -> &mut Self {
        self.when_disabled_prevent = Some(None);
        self
    }

    pub const fn build(self) -> IncrementingCounter {
        IncrementingCounter {
            auto_triggered_by: self.auto_triggered_by.unwrap(),
            trigger_target: self.trigger_target.unwrap(),
            when_target_reached: self.when_target_reached.unwrap(),
            when_disabled_prevent: self.when_disabled_prevent
                .expect("when_disable_prevent() must be set. For IRQs that can't be disabled, use never_disabled()."),
            ticking_enabled: true,
            triggering_enabled: true,
            count: 0,
        }
    }
}

#[derive(Clone, Copy)]
pub enum IncAutoTriggeredBy {
    AlreadyOnTarget,
    EndingOnTarget,
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum WhenTargetReached {
    Stay,
    Clear,
    Continue,
}