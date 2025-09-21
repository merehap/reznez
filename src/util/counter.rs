use std::sync::LazyLock;

pub struct DecrementingCounter {
    // Immutable settings determined at compile time
    auto_reload: bool,
    decrement_size: u16,
    decrementer: &'static LazyLock<Box<dyn DecrementingBehavior + Send + Sync + 'static>>,

    // State
    triggering_enabled: bool,
    reload_value: u16,
    count: u16,
    forced_reload_pending: bool,
}

impl DecrementingCounter {
    pub fn enabled(&self) -> bool {
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
        self.forced_reload_pending = true;
        self.count = self.reload_value;
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

        if counter.count == 0 && counter.auto_reload {
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

        counter.count = if already_on_zero && counter.auto_reload {
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
    reload_when_triggered: Option<bool>,
    initial_reload_value: Option<u16>,
    decrement_size: u16,
}

impl DecrementingCounterBuilder {
    pub const fn new() -> Self {
        Self {
            trigger_when: None,
            reload_when_triggered: None,
            initial_reload_value: None,
            decrement_size: 1,
        }
    }

    pub const fn trigger_when(&mut self, trigger_when: TriggerWhen) -> &mut Self {
        self.trigger_when = Some(trigger_when);
        self
    }

    pub const fn reload_when_triggered(&mut self, reload_when_triggered: bool) -> &mut Self {
        self.reload_when_triggered = Some(reload_when_triggered);
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
            auto_reload: self.reload_when_triggered.expect("reload_when_triggered must be set."),
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
pub enum TriggerWhen {
    DecrementingToZero,
    AlreadyZero,
}