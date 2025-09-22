use std::sync::LazyLock;

pub struct DecrementingCounter {
    // Immutable settings determined at compile time
    auto_reload: bool,
    forced_reload_behavior: ForcedReloadBehavior,
    decrement_size: u16,
    decrementer: &'static LazyLock<Box<dyn DecrementingBehavior + Send + Sync + 'static>>,

    // State
    triggering_enabled: bool,
    reload_value: u16,
    count: u16,
    forced_reload_pending: bool,
}

impl DecrementingCounter {
    pub fn triggering_enabled(&self) -> bool {
        self.triggering_enabled
    }

    pub fn enable_triggering(&mut self) {
        self.triggering_enabled = true;
    }

    pub fn disable_triggering(&mut self) {
        self.triggering_enabled = false;
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
            ForcedReloadBehavior::Immediate => self.count = self.reload_value,
            ForcedReloadBehavior::OnNextTick => self.forced_reload_pending = true,
        }
    }

    pub fn tick(&mut self) -> bool {
        let triggered = self.decrementer.decrement(self);
        self.forced_reload_pending = false;
        triggered
    }
}

trait DecrementingBehavior {
    fn decrement(&self, counter: &mut DecrementingCounter) -> bool;
}

static ANY_TRANSITION_TO_ZERO: LazyLock<Box<dyn DecrementingBehavior + Send + Sync>> = LazyLock::new(|| Box::new(TriggerOnAnyTransitionToZero));
static ONE_TO_ZERO_TRANSITION: LazyLock<Box<dyn DecrementingBehavior + Send + Sync>> = LazyLock::new(|| Box::new(TriggerOnOneToZeroTransition));
static ALREADY_ZERO: LazyLock<Box<dyn DecrementingBehavior + Send + Sync>> = LazyLock::new(|| Box::new(TriggerOnAlreadyZero));

struct TriggerOnAnyTransitionToZero;

impl DecrementingBehavior for TriggerOnAnyTransitionToZero {
    fn decrement(&self, counter: &mut DecrementingCounter) -> bool {
        let zero_counter_reload = counter.count == 0 && counter.auto_reload;
        counter.count = if zero_counter_reload || counter.forced_reload_pending {
            counter.reload_value
        } else {
            counter.count.saturating_sub(counter.decrement_size)
        };

        let triggered = counter.triggering_enabled && counter.count == 0;
        triggered
    }
}

struct TriggerOnOneToZeroTransition;

impl DecrementingBehavior for TriggerOnOneToZeroTransition {
    fn decrement(&self, counter: &mut DecrementingCounter) -> bool {
        let zero_counter_reload = counter.count == 0 && counter.auto_reload;
        let old_count = counter.count;
        counter.count = if zero_counter_reload || counter.forced_reload_pending {
            counter.reload_value
        } else {
            counter.count.saturating_sub(counter.decrement_size)
        };

        let triggered = counter.triggering_enabled && old_count == 1 && counter.count == 0;
        triggered
    }
}

struct TriggerOnAlreadyZero;

impl DecrementingBehavior for TriggerOnAlreadyZero {
    fn decrement(&self, counter: &mut DecrementingCounter) -> bool {
        // TODO: Determine if a forced reload needs to clear the counter before the actual reload occurs.
        // Some documentation claims this. This would only be relevant for AlreadyZero behavior since it
        // affects whether the counter is triggered or not during a forced reload.
        let triggered = counter.triggering_enabled && counter.count == 0;
        let zero_counter_reload = counter.auto_reload && counter.count == 0;
        counter.count = if zero_counter_reload || counter.forced_reload_pending {
            counter.reload_value
        } else {
            counter.count.saturating_sub(counter.decrement_size)
        };

        triggered
    }
}

#[derive(Clone, Copy)]
pub struct DecrementingCounterBuilder {
    trigger_on: Option<TriggerOn>,
    auto_reload: Option<bool>,
    forced_reload_behavior: Option<ForcedReloadBehavior>,
    initial_reload_value: u16,
    decrement_size: u16,
}

impl DecrementingCounterBuilder {
    pub const fn new() -> Self {
        Self {
            trigger_on: None,
            auto_reload: None,
            forced_reload_behavior: None,
            initial_reload_value: 0,
            decrement_size: 1,
        }
    }

    pub const fn trigger_on(&mut self, trigger_on: TriggerOn) -> &mut Self {
        self.trigger_on = Some(trigger_on);
        self
    }

    pub const fn auto_reload(&mut self, auto_reload: bool) -> &mut Self {
        self.auto_reload = Some(auto_reload);
        self
    }

    pub const fn forced_reload_behavior(&mut self, forced_reload_behavior: ForcedReloadBehavior) -> &mut Self {
        self.forced_reload_behavior = Some(forced_reload_behavior);
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
            TriggerOn::AnyTransitionToZero => &ANY_TRANSITION_TO_ZERO,
            TriggerOn::OneToZeroTransition => &ONE_TO_ZERO_TRANSITION,
            TriggerOn::AlreadyZero => &ALREADY_ZERO,
        };

        DecrementingCounter {
            auto_reload: self.auto_reload.expect("auto_reload must be set."),
            forced_reload_behavior: self.forced_reload_behavior.expect("forced_reload_behavior must be set"),
            decrement_size: self.decrement_size,
            decrementer,
            triggering_enabled: false,
            reload_value,
            count: reload_value,
            forced_reload_pending: false,
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum TriggerOn {
    AnyTransitionToZero,
    OneToZeroTransition,
    AlreadyZero,
}

#[derive(Clone, Copy)]
pub enum ForcedReloadBehavior {
    //Disabled,
    Immediate,
    OnNextTick,
}