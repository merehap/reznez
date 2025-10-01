use std::num::NonZeroU8;

use crate::counter::irq_counter_info::IrqCounterInfo;
pub use crate::counter::when_disabled_prevent::WhenDisabledPrevent;

pub struct Counter {
    // Immutable settings determined at compile time
    direction: Direction,
    auto_triggered_by: AutoTriggeredBy,
    trigger_on_forced_reload_of_zero: bool,
    forced_reload_timing: ForcedReloadTiming,
    auto_reload: bool,
    when_disabled_prevent: WhenDisabledPrevent,
    target_count: u16,
    step_size: NonZeroU8,

    // State
    triggering_enabled: bool,
    ticking_enabled: bool,
    reload_value: u16,
    count: u16,
    forced_reload_pending: bool,
    forced_trigger_pending: bool,
    prescaler: Prescaler,
}

impl Counter {
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
        if self.prescaler.behavior_on_forced_reload == PrescalerBehaviorOnForcedReload::ClearCount {
            self.prescaler.count = 0;
        }

        match self.forced_reload_timing {
            ForcedReloadTiming::Immediate => {
                self.count = self.reload_value;
                // Untested behavior, not sure if it exists in the wild. Should forced_trigger_pending be set if !triggering_enabled?
                if self.trigger_on_forced_reload_of_zero && self.reload_value == 0 {
                    self.forced_trigger_pending = true;
                }
            }
            ForcedReloadTiming::OnNextTick => self.forced_reload_pending = true,
        }
    }

    pub fn tick(&mut self) -> bool {
        let old_count = self.count;
        if self.ticking_enabled {
            // ASSUMPTION: Forced reloads and triggers (auto and forced) are prescaler-delayed, not just actual counter ticks.
            // NOTE: It's not clear how a counter that can only disable ticking can support a prescaler, so that fails compile.
            let prescaler_triggered = self.prescaler.tick();
            if !prescaler_triggered {
                // The prescaler didn't trigger yet, so don't tick nor trigger the actual counter.
                return false;
            }

            let zero_counter_reload = old_count == 0 && self.auto_reload;
            let should_reload = zero_counter_reload || self.forced_reload_pending;
            self.count = if should_reload {
                self.reload_value
            } else {
                let new_count = old_count.saturating_add_signed(self.direction as i16 * self.step_size.get() as i16);
                if old_count != self.target_count && new_count != self.target_count {
                    assert_eq!(new_count > self.target_count, old_count > self.target_count, "Stepped OVER the target count.");
                }

                new_count
            };
        }

        let new_count = self.count;
        // The triggering behavior is fixed at compile time, so the same branch will be taken every time here.
        let auto_triggered = match self.auto_triggered_by {
            AutoTriggeredBy::AlreadyOnTarget => old_count == self.target_count,
            AutoTriggeredBy::EndingOnTarget => new_count == self.target_count,
            AutoTriggeredBy::StepSizeTransitionToTarget =>
                i32::from(self.target_count) - i32::from(old_count) == self.direction as i32 * i32::from(self.step_size.get())
                    && new_count == self.target_count,
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

    pub fn to_irq_counter_info(&self) -> IrqCounterInfo {
        IrqCounterInfo {
            ticking_enabled: self.ticking_enabled,
            triggering_enabled: self.triggering_enabled,
            count: self.count,
        }
    }
}

// A decrementing counter where the count can be set directly, and can't be force-reloaded.
pub struct DirectlySetCounter(Counter);

impl DirectlySetCounter {
    // Used instead of set_reload_value_low_byte().
    pub fn set_count_low_byte(&mut self, value: u8) {
        self.0.count = (self.0.count & 0xFF00) | u16::from(value);
    }

    // Used instead of set_reload_value_high_byte().
    pub fn set_count_high_byte(&mut self, value: u8) {
        self.0.count = (self.0.count & 0x00FF) | (u16::from(value) << 8);
    }

    // force_reload() intentionally omitted from this list.
    pub fn enable(&mut self) { self.0.enable(); }
    pub fn disable(&mut self) { self.0.disable(); }
    pub fn tick(&mut self) -> bool { self.0.tick() }
    pub fn to_irq_counter_info(&self) -> IrqCounterInfo { self.0.to_irq_counter_info() }

    // The vast majority of use-cases should just call enable/disable instead of this.
    pub fn set_triggering_enabled(&mut self, triggering_enabled: bool) {
        self.0.triggering_enabled = triggering_enabled;
    }

    // The vast majority of use-cases should just call enable/disable instead of this.
    pub fn set_ticking_enabled(&mut self, ticking_enabled: bool) {
        self.0.ticking_enabled = ticking_enabled;
    }
}

#[derive(Clone, Copy)]
pub struct CounterBuilder {
    direction: Option<Direction>,
    auto_triggered_by: Option<AutoTriggeredBy>,
    trigger_on_forced_reload_of_zero: bool,
    auto_reload: Option<bool>,
    forced_reload_timing: Option<ForcedReloadTiming>,
    when_disabled_prevent: Option<WhenDisabledPrevent>,
    initial_reload_value: u16,
    initial_count: Option<u16>,
    step_size: NonZeroU8,
    prescaler: Prescaler,
}

impl CounterBuilder {
    pub const fn new() -> Self {
        Self {
            direction: None,
            auto_triggered_by: None,
            trigger_on_forced_reload_of_zero: false,
            auto_reload: None,
            forced_reload_timing: None,
            when_disabled_prevent: None,
            initial_reload_value: 0,
            // Normally initial_reload_value is assigned to initial_count in build().
            initial_count: None,
            step_size: NonZeroU8::new(1).unwrap(),
            // A prescaler that doesn't actually delay anything.
            prescaler: Prescaler::DEFAULT,
        }
    }

    pub const fn direction(&mut self, direction: Direction) -> &mut Self {
        self.direction = Some(direction);
        self
    }

    pub const fn auto_triggered_by(&mut self, auto_triggered_by: AutoTriggeredBy) -> &mut Self {
        self.auto_triggered_by = Some(auto_triggered_by);
        self
    }

    pub const fn also_trigger_on_forced_reload_of_zero(&mut self) -> &mut Self {
        self.trigger_on_forced_reload_of_zero = true;
        self
    }

    pub const fn on_forced_reload_set_count(&mut self, forced_reload_behavior: ForcedReloadTiming) -> &mut Self {
        self.forced_reload_timing = Some(forced_reload_behavior);
        self
    }

    pub const fn auto_reload(&mut self, auto_reload: bool) -> &mut Self {
        self.auto_reload = Some(auto_reload);
        self
    }

    pub const fn when_disabled_prevent(&mut self, when_disabled_prevent: WhenDisabledPrevent) -> &mut Self {
        self.when_disabled_prevent = Some(when_disabled_prevent);
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

    pub const fn step_size(&mut self, size: u8) -> &mut Self {
        self.step_size = NonZeroU8::new(size).expect("decrement_size must be positive");
        self
    }

    pub const fn prescaler(
        &mut self,
        multiple: u8,
        prescaler_triggered_by: PrescalerTriggeredBy,
        prescaler_behavior_on_forced_reload: PrescalerBehaviorOnForcedReload,
    ) -> &mut Self {
        self.prescaler = Prescaler {
            multiple: NonZeroU8::new(multiple).expect("prescaler multiple must be positive"),
            triggered_by: prescaler_triggered_by,
            behavior_on_forced_reload: prescaler_behavior_on_forced_reload,
            count: 0,
        };
        self
    }

    pub const fn build(self) -> Counter {
        assert!(self.forced_reload_timing.is_some(),
            "forced_reload_timing must be set. If forced-reloading is not needed, use build_directly_settable() instead.");
        self.build_reload_forceable()
    }

    pub const fn build_directly_set(mut self) -> DirectlySetCounter {
        // Set an unused dummy value so validation will pass.
        self.forced_reload_timing = Some(ForcedReloadTiming::Immediate);
        DirectlySetCounter(self.build_reload_forceable())
    }

    const fn build_reload_forceable(self) -> Counter {
        let auto_triggered_by = self.auto_triggered_by.expect("auto_triggered_by must be set");
        if matches!(auto_triggered_by, AutoTriggeredBy::StepSizeTransitionToTarget) && self.step_size.get() > 1 {
            panic!("AutoTriggeredBy::OneToZeroTransition must not be specified when decrement_size is greater than 1.");
        }

        let reload_value = self.initial_reload_value;
        let when_disabled_prevent = self.when_disabled_prevent.expect("when_disabled must be set");
        let ticking_enabled = match when_disabled_prevent {
            // Counters that CANNOT disable ticking will always have ticking enabled.
            WhenDisabledPrevent::Triggering => true,
            // Counters that CAN disable ticking should START with ticking disabled.
            WhenDisabledPrevent::Ticking | WhenDisabledPrevent::TickingAndTriggering => false,
        };

        if matches!(when_disabled_prevent, WhenDisabledPrevent::Ticking) && !self.prescaler.is_nop() {
            panic!("WhenDisabledPrevent::Ticking must not be specified at the same as a prescaler.");
        }

        Counter {
            direction: self.direction.expect("direction must be set"),
            auto_triggered_by,
            trigger_on_forced_reload_of_zero: self.trigger_on_forced_reload_of_zero,
            forced_reload_timing: self.forced_reload_timing.expect("forced_reload_timing must be set"),
            auto_reload: self.auto_reload.expect("auto_reload must be set"),
            when_disabled_prevent,
            target_count: 0,
            step_size: self.step_size,
            prescaler: self.prescaler,
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
pub enum Direction {
    Decrementing = -1,
    Incrementing = 1,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum AutoTriggeredBy {
    AlreadyOnTarget,
    EndingOnTarget,
    StepSizeTransitionToTarget,
}

// OnForcedReloadSetCount
//
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum ForcedReloadTiming {
    Immediate,
    OnNextTick,
}

#[derive(Clone, Copy, Debug)]
struct Prescaler {
    // Immutable settings determined at compile time
    multiple: NonZeroU8,
    triggered_by: PrescalerTriggeredBy,
    behavior_on_forced_reload: PrescalerBehaviorOnForcedReload,

    // State
    count: u8,
}

impl Prescaler {
    // A multiple of 1 effectively means the prescaler has no effect, regardless of which triggered_by behavior is used.
    const DEFAULT: Self = Self {
        multiple: NonZeroU8::new(1).unwrap(),
        triggered_by: PrescalerTriggeredBy::AlreadyZero,
        behavior_on_forced_reload: PrescalerBehaviorOnForcedReload::DoNothing,
        count: 0,
    };

    fn tick(&mut self) -> bool {
        let old_count = self.count;
        self.count += 1;
        self.count %= self.multiple;
        let new_count = self.count;

        let triggered = match self.triggered_by {
            PrescalerTriggeredBy::AlreadyZero => old_count == 0,
            PrescalerTriggeredBy::WrappingToZero => new_count == 0,
        };

        triggered
    }

    const fn is_nop(&self) -> bool {
        self.multiple.get() == 1
    }
}

#[derive(Clone, Copy, Debug)]
pub enum PrescalerTriggeredBy {
    AlreadyZero,
    WrappingToZero,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum PrescalerBehaviorOnForcedReload {
    DoNothing,
    ClearCount,
}