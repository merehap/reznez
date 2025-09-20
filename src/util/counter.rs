pub struct DecrementingCounter {
    enabled: bool,
    disabled_behavior: DisabledBehavior,
    trigger_when: TriggerWhen,
    reload_when_triggered: bool,
    count: u16,
    reload_value: u16,
    decrement_size: u16,
}

impl DecrementingCounter {
    pub fn enabled(&self) -> bool {
        self.enabled
    }

    pub fn enable(&mut self) {
        self.enabled = true;
    }

    pub fn disable(&mut self) {
        self.enabled = false;
    }

    pub fn set_reload_value_low_byte(&mut self, value: u8) {
        self.reload_value = (self.reload_value & 0xFF00) | u16::from(value);
    }

    pub fn set_reload_value_high_byte(&mut self, value: u8) {
        self.reload_value = (self.reload_value & 0x00FF) | (u16::from(value) << 8);
    }

    pub fn force_reload(&mut self) {
        self.count = self.reload_value;
    }

    pub fn decrement(&mut self) -> bool {
        if !self.enabled && self.disabled_behavior == DisabledBehavior::DoNothing {
            return false;
        }

        let mut maybe_triggered = false;
        match self.trigger_when {
            TriggerWhen::AlreadyZero => {
                if self.count == 0 {
                    maybe_triggered = true;
                } else {
                    self.count = self.count.saturating_sub(self.decrement_size);
                }
            }
            TriggerWhen::DecrementedToZero => {
                self.count = self.count.saturating_sub(self.decrement_size);
                if self.count == 0 {
                    maybe_triggered = true;
                }
            }
        }

        if maybe_triggered && self.reload_when_triggered {
            self.count = self.reload_value;
        }

        let triggered = self.enabled && maybe_triggered;
        triggered
    }
}

#[derive(Clone, Copy)]
pub struct DecrementingCounterBuilder {
    disabled_behavior: Option<DisabledBehavior>,
    trigger_when: Option<TriggerWhen>,
    reload_when_triggered: Option<bool>,
    initial_reload_value: Option<u16>,
    decrement_size: u16,
}

impl DecrementingCounterBuilder {
    pub const fn new() -> Self {
        Self {
            disabled_behavior: None,
            trigger_when: None,
            reload_when_triggered: None,
            initial_reload_value: None,
            decrement_size: 1,
        }
    }

    pub const fn when_disabled(&mut self, disabled_behavior: DisabledBehavior) -> &mut Self {
        self.disabled_behavior = Some(disabled_behavior);
        self
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
        DecrementingCounter {
            enabled: false,
            disabled_behavior: self.disabled_behavior.expect("when_disabled must be set."),
            trigger_when: self.trigger_when.expect("trigger_when must be set."),
            reload_when_triggered: self.reload_when_triggered.expect("reload_when_triggered must be set."),
            count: reload_value,
            reload_value,
            decrement_size: self.decrement_size,
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum DisabledBehavior {
    DoNothing,
    Tick,
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum TriggerWhen {
    DecrementedToZero,
    AlreadyZero,
}