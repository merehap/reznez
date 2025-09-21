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
        }
    }

    pub fn decrement(&mut self) -> bool {
        self.decrementer.decrement(self)
    }
}

trait DecrementingBehavior {
    fn decrement(&self, counter: &mut DecrementingCounter) -> bool;
}

static DECREMENTING_TO_ZERO: LazyLock<Box<dyn DecrementingBehavior + Send + Sync>> = LazyLock::new(|| Box::new(TriggerOnDecrementingToZero));
static ALREADY_ZERO: LazyLock<Box<dyn DecrementingBehavior + Send + Sync>> = LazyLock::new(|| Box::new(TriggerOnAlreadyZero));

struct TriggerOnDecrementingToZero;

impl DecrementingBehavior for TriggerOnDecrementingToZero {
    fn decrement(&self, counter: &mut DecrementingCounter) -> bool {
        counter.count = counter.count.saturating_sub(counter.decrement_size);
        let should_auto_reload = counter.count == 0 && counter.auto_reload;
        if should_auto_reload {
            counter.count = counter.reload_value;
        }

        let triggered = counter.count == 0 && counter.triggering_enabled;
        triggered
    }
}

struct TriggerOnAlreadyZero;

impl DecrementingBehavior for TriggerOnAlreadyZero {
    fn decrement(&self, counter: &mut DecrementingCounter) -> bool {
        let already_on_zero = counter.count == 0;
        counter.count = if counter.auto_reload && already_on_zero {
            counter.reload_value
        } else {
            counter.count.saturating_sub(counter.decrement_size)
        };

        let triggered = already_on_zero && counter.triggering_enabled;
        triggered
    }
}

#[derive(Clone, Copy)]
pub struct DecrementingCounterBuilder {
    trigger_when: Option<TriggerWhen>,
    auto_reload: Option<bool>,
    forced_reload_behavior: Option<ForcedReloadBehavior>,
    initial_reload_value: Option<u16>,
    decrement_size: u16,
}

impl DecrementingCounterBuilder {
    pub const fn new() -> Self {
        Self {
            trigger_when: None,
            auto_reload: None,
            forced_reload_behavior: None,
            initial_reload_value: None,
            decrement_size: 1,
        }
    }

    pub const fn trigger_when(&mut self, trigger_when: TriggerWhen) -> &mut Self {
        self.trigger_when = Some(trigger_when);
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
        self.initial_reload_value = Some(value);
        self
    }

    pub const fn decrement_size(&mut self, size: u16) -> &mut Self {
        self.decrement_size = size;
        self
    }

    pub const fn build(self) -> DecrementingCounter {
        let reload_value = self.initial_reload_value.expect("initial_counter_reload_value must be set.");
        let decrementer: &LazyLock<Box<dyn DecrementingBehavior + Send + Sync + 'static>> = match self.trigger_when.expect("trigger_when must be set") {
            TriggerWhen::DecrementingToZero => &DECREMENTING_TO_ZERO,
            TriggerWhen::AlreadyZero => &ALREADY_ZERO,
        };

        DecrementingCounter {
            auto_reload: self.auto_reload.expect("auto_reload must be set."),
            forced_reload_behavior: self.forced_reload_behavior.expect("forced_reload_behavior must be set"),
            decrement_size: self.decrement_size,
            decrementer,
            triggering_enabled: false,
            reload_value,
            count: reload_value,
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum TriggerWhen {
    DecrementingToZero,
    AlreadyZero,
}

#[derive(Clone, Copy)]
pub enum ForcedReloadBehavior {
    //Disabled,
    Immediate
}