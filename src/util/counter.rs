use std::sync::LazyLock;

pub struct DecrementingCounter {
    // Immutable settings determined at compile time
    trigger_on_forced_reload_of_zero: bool,
    forced_reload_behavior: ForcedReloadBehavior,
    auto_reload: bool,
    when_disabled: WhenDisabled,
    decrement_size: u16,
    decrementer: &'static LazyLock<Box<dyn DecrementingBehavior + Send + Sync + 'static>>,

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
        match self.when_disabled {
            WhenDisabled::PreventTriggering => self.triggering_enabled = false,
            WhenDisabled::PreventTicking => self.ticking_enabled = false,
        }
    }

    pub fn set_reload_value(&mut self, value: u8) {
        self.reload_value = u16::from(value);
    }

    pub fn set_reload_value_low_byte(&mut self, value: u8) {
        self.reload_value = (self.reload_value & 0xFF00) | u16::from(value);
    }

    pub fn set_reload_value_high_byte(&mut self, value: u8) {
        self.reload_value = (self.reload_value & 0x00FF) | (u16::from(value) << 8);
    }

    pub fn force_reload(&mut self) {
        match self.forced_reload_behavior {
            //ForcedReloadBehavior::Disabled => panic!("forced_reload_timing must be specified in DecrementingCounterBuilder in order to call forced_reload"),
            ForcedReloadBehavior::Immediate => {
                self.count = self.reload_value;
                // Untested behavior, not sure if it exists in the wild. Should forced_trigger_pending be set if !triggering_enabled?
                if self.trigger_on_forced_reload_of_zero && self.reload_value == 0 {
                    self.forced_trigger_pending = true;
                }
            }
            ForcedReloadBehavior::OnNextTick => self.forced_reload_pending = true,
        }
    }

    pub fn tick(&mut self) -> bool {
        let triggered = self.decrementer.decrement(self);
        self.forced_reload_pending = false;
        self.forced_trigger_pending = false;
        triggered
    }
}

trait DecrementingBehavior {
    fn decrement(&self, counter: &mut DecrementingCounter) -> bool;
}

static ENDING_ON_ZERO: LazyLock<Box<dyn DecrementingBehavior + Send + Sync>> = LazyLock::new(|| Box::new(TriggerOnTransitionToZero));
static ONE_TO_ZERO_TRANSITION: LazyLock<Box<dyn DecrementingBehavior + Send + Sync>> = LazyLock::new(|| Box::new(TriggerOnOneToZeroTransition));
static ALREADY_ZERO: LazyLock<Box<dyn DecrementingBehavior + Send + Sync>> = LazyLock::new(|| Box::new(TriggerOnAlreadyZero));

struct TriggerOnTransitionToZero;

impl DecrementingBehavior for TriggerOnTransitionToZero {
    fn decrement(&self, counter: &mut DecrementingCounter) -> bool {
        if counter.ticking_enabled {
            let zero_counter_reload = counter.count == 0 && counter.auto_reload;
            let should_reload = zero_counter_reload || counter.forced_reload_pending;
            counter.count = if should_reload {
                counter.reload_value
            } else {
                counter.count.saturating_sub(counter.decrement_size)
            };
        }

        let triggered_by_zero_result = counter.count == 0;
        let mut triggered_by_forcing = counter.trigger_on_forced_reload_of_zero && counter.forced_reload_pending && counter.reload_value == 0;
        triggered_by_forcing |= counter.forced_trigger_pending;
        let trigger_if_enabled = triggered_by_zero_result || triggered_by_forcing;
        trigger_if_enabled && counter.triggering_enabled
    }
}

struct TriggerOnOneToZeroTransition;

impl DecrementingBehavior for TriggerOnOneToZeroTransition {
    fn decrement(&self, counter: &mut DecrementingCounter) -> bool {
        let old_count = counter.count;

        if counter.ticking_enabled {
            let zero_counter_reload = old_count == 0 && counter.auto_reload;
            let should_reload = zero_counter_reload || counter.forced_reload_pending;
            counter.count = if should_reload {
                counter.reload_value
            } else {
                counter.count.saturating_sub(counter.decrement_size)
            };
        }

        let triggered_by_one_to_zero_transition = old_count == 1 && counter.count == 0;
        let mut triggered_by_forcing = counter.trigger_on_forced_reload_of_zero && counter.forced_reload_pending && counter.reload_value == 0;
        triggered_by_forcing |= counter.forced_trigger_pending;
        let trigger_if_enabled = triggered_by_one_to_zero_transition || triggered_by_forcing;
        trigger_if_enabled && counter.triggering_enabled
    }
}

struct TriggerOnAlreadyZero;

impl DecrementingBehavior for TriggerOnAlreadyZero {
    fn decrement(&self, counter: &mut DecrementingCounter) -> bool {
        let triggered_by_already_zero = counter.count == 0;

        if counter.ticking_enabled {
            let zero_counter_reload = counter.count == 0 && counter.auto_reload;
            let should_reload = zero_counter_reload || counter.forced_reload_pending;
            counter.count = if should_reload {
                counter.reload_value
            } else {
                counter.count.saturating_sub(counter.decrement_size)
            };
        }

        // TODO: Determine if a forced reload needs to clear the counter before the reloading actually occurs.
        // Some documentation claims this. This would only be relevant for AlreadyZero behavior since it
        // affects whether the counter is triggered or not during a forced reload.
        let mut triggered_by_forcing = counter.trigger_on_forced_reload_of_zero && counter.forced_reload_pending && counter.reload_value == 0;
        triggered_by_forcing |= counter.forced_trigger_pending;
        let trigger_if_enabled = triggered_by_already_zero || triggered_by_forcing;
        trigger_if_enabled && counter.triggering_enabled
    }
}

#[derive(Clone, Copy)]
pub struct DecrementingCounterBuilder {
    trigger_on: Option<TriggerOn>,
    trigger_on_forced_reload_of_zero: bool,
    auto_reload: Option<bool>,
    forced_reload_behavior: Option<ForcedReloadBehavior>,
    when_disabled: Option<WhenDisabled>,
    initial_reload_value: u16,
    decrement_size: u16,
}

impl DecrementingCounterBuilder {
    pub const fn new() -> Self {
        Self {
            trigger_on: None,
            trigger_on_forced_reload_of_zero: false,
            auto_reload: None,
            forced_reload_behavior: None,
            when_disabled: None,
            initial_reload_value: 0,
            decrement_size: 1,
        }
    }

    pub const fn trigger_on(&mut self, trigger_on: TriggerOn) -> &mut Self {
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

    pub const fn when_disabled(&mut self, when_disabled: WhenDisabled) -> &mut Self {
        self.when_disabled = Some(when_disabled);
        self
    }

    pub const fn initial_reload_value(&mut self, value: u16) -> &mut Self {
        self.initial_reload_value = value;
        self
    }

    pub const fn decrement_size(&mut self, size: u16) -> &mut Self {
        self.decrement_size = size;
        self
    }

    pub const fn build(self) -> DecrementingCounter {
        let reload_value = self.initial_reload_value;
        let decrementer: &LazyLock<Box<dyn DecrementingBehavior + Send + Sync + 'static>> = match self.trigger_on.expect("trigger_when must be set") {
            TriggerOn::EndingOnZero => &ENDING_ON_ZERO,
            TriggerOn::OneToZeroTransition => &ONE_TO_ZERO_TRANSITION,
            TriggerOn::AlreadyZero => &ALREADY_ZERO,
        };

        let when_disabled = self.when_disabled.expect("when_disabled must be set");
        let ticking_enabled = match when_disabled {
            // Counters that CANNOT disable ticking will always have ticking enabled.
            WhenDisabled::PreventTriggering => true,
            // Counters that CAN disable ticking should START with ticking disabled.
            WhenDisabled::PreventTicking => false,
        };

        DecrementingCounter {
            forced_reload_behavior: self.forced_reload_behavior.expect("forced_reload_behavior must be set"),
            trigger_on_forced_reload_of_zero: self.trigger_on_forced_reload_of_zero,
            auto_reload: self.auto_reload.expect("auto_reload must be set"),
            when_disabled,
            decrement_size: self.decrement_size,
            decrementer,
            triggering_enabled: false,
            ticking_enabled,
            reload_value,
            count: reload_value,
            forced_reload_pending: false,
            forced_trigger_pending: false,
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum TriggerOn {
    EndingOnZero,
    OneToZeroTransition,
    AlreadyZero,
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum ForcedReloadBehavior {
    //Disabled,
    Immediate,
    OnNextTick,
}

#[derive(Clone, Copy)]
pub enum WhenDisabled {
    PreventTriggering,
    PreventTicking,
}