pub struct DecrementingCounter {
    // Immutable settings determined at compile time
    trigger_on: AutoTriggeredBy,
    trigger_on_forced_reload_of_zero: bool,
    forced_reload_behavior: ForcedReloadBehavior,
    auto_reload: bool,
    when_disabled_prevent: WhenDisabledPrevent,
    decrement_size: u16,

    // State
    triggering_enabled: bool,
    ticking_enabled: bool,
    reload_value: u16,
    count: u16,
    forced_reload_pending: bool,
    forced_trigger_pending: bool,
}

impl DecrementingCounter {
    // TODO: Try to stop exposing this publicly.
    pub fn triggering_enabled(&self) -> bool {
        self.triggering_enabled
    }

    pub fn enable(&mut self) {
        self.triggering_enabled = true;
        self.ticking_enabled = true;
    }

    pub fn disable(&mut self) {
        match self.when_disabled_prevent {
            WhenDisabledPrevent::Ticking => self.ticking_enabled = false,
            WhenDisabledPrevent::Triggering => self.triggering_enabled = false,
            WhenDisabledPrevent::TickingAndTriggering => {
                self.ticking_enabled = false;
                self.triggering_enabled = false;
            }
        }
    }

    // The vast majority of use-cases should just call enable/disable instead of this.
    pub fn set_triggering_enabled(&mut self, triggering_enabled: bool) {
        self.triggering_enabled = triggering_enabled;
    }

    // The vast majority of use-cases should just call enable/disable instead of this.
    pub fn set_ticking_enabled(&mut self, ticking_enabled: bool) {
        self.ticking_enabled = ticking_enabled;
    }

    pub fn set_reload_value(&mut self, value: u8) {
        assert!(self.forced_reload_behavior != ForcedReloadBehavior::SetCountDirectly,
            "When forced_reload_behavior == DirectlySetCount, use set_count_X() instead of set_reload_value_X()");
        self.reload_value = u16::from(value);
    }

    pub fn set_reload_value_low_byte(&mut self, value: u8) {
        assert!(self.forced_reload_behavior != ForcedReloadBehavior::SetCountDirectly,
            "When forced_reload_behavior == DirectlySetCount, use set_count_X() instead of set_reload_value_X()");
        self.reload_value = (self.reload_value & 0xFF00) | u16::from(value);
    }

    pub fn set_reload_value_high_byte(&mut self, value: u8) {
        assert!(self.forced_reload_behavior != ForcedReloadBehavior::SetCountDirectly,
            "When forced_reload_behavior == DirectlySetCount, use set_count_X() instead of set_reload_value_X()");
        self.reload_value = (self.reload_value & 0x00FF) | (u16::from(value) << 8);
    }

    pub fn set_count_low_byte(&mut self, value: u8) {
        assert_eq!(self.forced_reload_behavior, ForcedReloadBehavior::SetCountDirectly,
            "Must use forced_reload_behavior == DirectlySetCount in order to call set_count_X()");
        self.count = (self.count & 0xFF00) | u16::from(value);
    }

    pub fn set_count_high_byte(&mut self, value: u8) {
        assert_eq!(self.forced_reload_behavior, ForcedReloadBehavior::SetCountDirectly,
            "Must use forced_reload_behavior == DirectlySetCount in order to call set_count_X()");
        self.count = (self.count & 0x00FF) | (u16::from(value) << 8);
    }

    pub fn force_reload(&mut self) {
        match self.forced_reload_behavior {
            ForcedReloadBehavior::SetCountDirectly => panic!("forced_reload_timing must be specified in DecrementingCounterBuilder in order to call forced_reload"),
            ForcedReloadBehavior::SetReloadValueImmediately => {
                self.count = self.reload_value;
                // Untested behavior, not sure if it exists in the wild. Should forced_trigger_pending be set if !triggering_enabled?
                if self.trigger_on_forced_reload_of_zero && self.reload_value == 0 {
                    self.forced_trigger_pending = true;
                }
            }
            ForcedReloadBehavior::SetReloadValueOnNextTick => self.forced_reload_pending = true,
        }
    }

    pub fn tick(&mut self) -> bool {
        let old_count = self.count;
        if self.ticking_enabled {
            let zero_counter_reload = old_count == 0 && self.auto_reload;
            let should_reload = zero_counter_reload || self.forced_reload_pending;
            self.count = if should_reload {
                self.reload_value
            } else {
                self.count.saturating_sub(self.decrement_size)
            };
        }

        let new_count = self.count;
        // The triggering behavior is fixed at compile time, so the same branch will be taken every time here.
        let auto_triggered = match self.trigger_on {
            AutoTriggeredBy::AlreadyZero => old_count == 0,
            AutoTriggeredBy::EndingOnZero => new_count == 0,
            AutoTriggeredBy::OneToZeroTransition => old_count == 1 && new_count == 0,
        };
        // TODO: Determine if a forced reload needs to clear the counter before the reloading actually occurs for some cases.
        // Some documentation claims this. This would only be relevant for AlreadyZero behavior since it
        // affects whether the counter is triggered or not during a forced reload.
        let mut triggered_by_forcing = self.trigger_on_forced_reload_of_zero && self.forced_reload_pending && self.reload_value == 0;
        triggered_by_forcing |= self.forced_trigger_pending;

        let trigger_if_enabled = auto_triggered || triggered_by_forcing;
        let triggered = trigger_if_enabled && self.triggering_enabled;

        self.forced_reload_pending = false;
        self.forced_trigger_pending = false;
        triggered
    }
}

#[derive(Clone, Copy)]
pub struct DecrementingCounterBuilder {
    trigger_on: Option<AutoTriggeredBy>,
    trigger_on_forced_reload_of_zero: bool,
    auto_reload: Option<bool>,
    forced_reload_behavior: Option<ForcedReloadBehavior>,
    when_disabled_prevent: Option<WhenDisabledPrevent>,
    initial_reload_value: u16,
    initial_count: Option<u16>,
    decrement_size: u16,
}

impl DecrementingCounterBuilder {
    pub const fn new() -> Self {
        Self {
            trigger_on: None,
            trigger_on_forced_reload_of_zero: false,
            auto_reload: None,
            forced_reload_behavior: None,
            when_disabled_prevent: None,
            initial_reload_value: 0,
            // Normally initial_reload_value is assigned to initial_count in build().
            initial_count: None,
            decrement_size: 1,
        }
    }

    pub const fn auto_trigger_on(&mut self, trigger_on: AutoTriggeredBy) -> &mut Self {
        self.trigger_on = Some(trigger_on);
        self
    }

    pub const fn also_trigger_on_forced_reload_of_zero(&mut self) -> &mut Self {
        self.trigger_on_forced_reload_of_zero = true;
        self
    }

    pub const fn forced_reload_behavior(&mut self, forced_reload_behavior: ForcedReloadBehavior) -> &mut Self {
        self.forced_reload_behavior = Some(forced_reload_behavior);
        self
    }

    pub const fn auto_reload(&mut self, auto_reload: bool) -> &mut Self {
        self.auto_reload = Some(auto_reload);
        self
    }

    pub const fn when_disabled_prevent(&mut self, when_disabled: WhenDisabledPrevent) -> &mut Self {
        self.when_disabled_prevent = Some(when_disabled);
        self
    }

    pub const fn initial_reload_value(&mut self, value: u16) -> &mut Self {
        self.initial_reload_value = value;
        self
    }

    pub const fn initial_count(&mut self, value: u16) -> &mut Self {
        self.initial_count = Some(value);
        self
    }

    pub const fn decrement_size(&mut self, size: u16) -> &mut Self {
        self.decrement_size = size;
        self
    }

    pub const fn build(self) -> DecrementingCounter {
        let reload_value = self.initial_reload_value;
        let when_disabled_prevent = self.when_disabled_prevent.expect("when_disabled must be set");
        let ticking_enabled = match when_disabled_prevent {
            // Counters that CANNOT disable ticking will always have ticking enabled.
            WhenDisabledPrevent::Triggering => true,
            // Counters that CAN disable ticking should START with ticking disabled.
            WhenDisabledPrevent::Ticking | WhenDisabledPrevent::TickingAndTriggering => false,
        };

        DecrementingCounter {
            trigger_on: self.trigger_on.expect("trigger_on must be set"),
            trigger_on_forced_reload_of_zero: self.trigger_on_forced_reload_of_zero,
            forced_reload_behavior: self.forced_reload_behavior.expect("forced_reload_behavior must be set"),
            auto_reload: self.auto_reload.expect("auto_reload must be set"),
            when_disabled_prevent,
            decrement_size: self.decrement_size,
            triggering_enabled: false,
            ticking_enabled,
            reload_value,
            count: self.initial_count.unwrap_or(reload_value),
            forced_reload_pending: false,
            forced_trigger_pending: false,
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum AutoTriggeredBy {
    AlreadyZero,
    EndingOnZero,
    OneToZeroTransition,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum ForcedReloadBehavior {
    SetCountDirectly,
    SetReloadValueImmediately,
    SetReloadValueOnNextTick,
}

#[derive(Clone, Copy, Debug)]
pub enum WhenDisabledPrevent {
    Ticking,
    Triggering,
    TickingAndTriggering,
}