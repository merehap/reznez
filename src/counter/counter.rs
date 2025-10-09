use std::num::{NonZeroI8, NonZeroU8};

use crate::counter::irq_counter_info::IrqCounterInfo;
pub use crate::counter::when_disabled_prevent::WhenDisabledPrevent;

// A counter where the count can be set directly, and can't be force-reloaded.
pub struct DirectlySetCounter(Counter);

impl DirectlySetCounter {
    pub fn enable(&mut self) { self.0.enable(); }
    pub fn disable(&mut self) { self.0.disable(); }
    pub fn tick(&mut self) -> TickResult { self.0.tick(false, false) }
    pub fn to_irq_counter_info(&self) -> IrqCounterInfo { self.0.to_irq_counter_info() }

    pub fn count_low_byte(&self) -> u8 {
        self.0.count.to_be_bytes()[1]
    }

    pub fn count_high_byte(&self) -> u8 {
        self.0.count.to_be_bytes()[0]
    }

    pub fn set_count_low_byte(&mut self, value: u8) {
        self.0.count = (self.0.count & 0xFF00) | u16::from(value);
    }

    pub fn set_count_high_byte(&mut self, value: u8) {
        self.0.count = (self.0.count & 0x00FF) | (u16::from(value) << 8);
    }

    // The vast majority of use-cases should just call enable/disable instead of this.
    pub fn set_triggering_enabled(&mut self, triggering_enabled: bool) {
        self.0.triggering_enabled = triggering_enabled;
    }

    // The vast majority of use-cases should just call enable/disable instead of this.
    pub fn set_ticking_enabled(&mut self, ticking_enabled: bool) {
        self.0.ticking_enabled = ticking_enabled;
    }
}

// A counter where the count only be set by reloading through a reload value.
pub struct ReloadDrivenCounter {
    counter: Counter,
    forced_reload_timing: ForcedReloadTiming,
    trigger_on_forced_reload_with_target_count: bool,
    forced_reload_pending: bool,
}

impl ReloadDrivenCounter {
    pub fn enable(&mut self) { self.counter.enable(); }
    pub fn disable(&mut self) { self.counter.disable(); }
    pub fn to_irq_counter_info(&self) -> IrqCounterInfo { self.counter.to_irq_counter_info() }

    pub fn set_reload_value(&mut self, value: u8) {
        self.counter.reload_value = u16::from(value);
    }

    pub fn set_reload_value_low_byte(&mut self, value: u8) {
        self.counter.reload_value = (self.counter.reload_value & 0xFF00) | u16::from(value);
    }

    pub fn set_reload_value_high_byte(&mut self, value: u8) {
        self.counter.reload_value = (self.counter.reload_value & 0x00FF) | (u16::from(value) << 8);
    }

    pub fn force_reload(&mut self) {
        if self.counter.prescaler.behavior_on_forced_reload == PrescalerBehaviorOnForcedReload::ClearCount {
            self.counter.prescaler.count = 0;
        }

        match self.forced_reload_timing {
            ForcedReloadTiming::Immediate => self.counter.count = self.counter.reload_value,
            ForcedReloadTiming::OnNextTick => self.forced_reload_pending = true,
        }
    }

    pub fn tick(&mut self) -> TickResult {
        // TODO: Determine if a forced reload needs to clear the counter before the reloading actually occurs for some cases.
        // Some documentation claims this. This would only be relevant for AlreadyZero behavior since it
        // affects whether the counter is triggered or not during a forced reload.
        let triggered_by_forcing = self.trigger_on_forced_reload_with_target_count
            && self.forced_reload_pending
            && self.counter.reload_value == self.counter.target_count;

        let result = self.counter.tick(self.forced_reload_pending, triggered_by_forcing);
        if !result.skipped {
            self.forced_reload_pending = false;
        }

        result
    }
}

struct Counter {
    // Immutable settings determined at compile time
    step: NonZeroI8,
    auto_triggered_by: AutoTriggeredBy,
    target_count: u16,
    when_target_reached: WhenTargetReached,
    when_disabled_prevent: WhenDisabledPrevent,

    // State
    triggering_enabled: bool,
    ticking_enabled: bool,
    reload_value: u16,
    count: u16,
    prescaler: Prescaler,
}

impl Counter {
    fn enable(&mut self) {
        self.triggering_enabled = true;
        self.ticking_enabled = true;
    }

    fn disable(&mut self) {
        match self.when_disabled_prevent {
            WhenDisabledPrevent::Ticking => self.ticking_enabled = false,
            WhenDisabledPrevent::Triggering => self.triggering_enabled = false,
            WhenDisabledPrevent::TickingAndTriggering => {
                self.ticking_enabled = false;
                self.triggering_enabled = false;
            }
        }
    }

    fn tick(&mut self, forced_reload_pending: bool, triggered_by_forcing: bool) -> TickResult {
        let old_count = self.count;
        let mut wrapped = false;
        if self.ticking_enabled {
            // ASSUMPTION: Forced reloads and triggers (auto and forced) are prescaler-delayed, not just actual counter ticks.
            // NOTE: It's not clear how a counter that can only disable ticking can support a prescaler, so that fails compile.
            let prescaler_triggered = self.prescaler.tick();
            if !prescaler_triggered {
                // The prescaler didn't trigger yet, so don't tick nor trigger the actual counter.
                return TickResult { skipped: true, wrapped: false, triggered: false };
            }

            let auto_reload = match self.when_target_reached {
                WhenTargetReached::Stay => false,
                WhenTargetReached::Reload => old_count == self.target_count,
                WhenTargetReached::ContinueThenReloadAfter(wrap_count) => old_count == wrap_count,
            };
            let stay_on_count = self.when_target_reached == WhenTargetReached::Stay && old_count == self.target_count;
            if auto_reload { 
                wrapped = true;
                self.count = self.reload_value;
            } else if forced_reload_pending {
                self.count = self.reload_value;
            } else if !stay_on_count {
                self.count = old_count.saturating_add_signed(self.step.get() as i16);
                if old_count != self.target_count && self.count != self.target_count {
                    assert_eq!(self.count > self.target_count, old_count > self.target_count, "Stepped OVER the target count.");
                }
            }
        }

        let new_count = self.count;
        // The triggering behavior is fixed at compile time, so the same branch will be taken every time here.
        let auto_triggered = match self.auto_triggered_by {
            AutoTriggeredBy::AlreadyOn => old_count == self.target_count,
            AutoTriggeredBy::EndingOn => new_count == self.target_count,
            AutoTriggeredBy::StepSizedTransitionTo =>
                i32::from(self.target_count) - i32::from(old_count) == i32::from(self.step.get())
                    && new_count == self.target_count,
        };

        let trigger_if_enabled = auto_triggered || triggered_by_forcing;
        let triggered = trigger_if_enabled && self.triggering_enabled;

        TickResult { skipped: false, wrapped, triggered }
    }

    fn to_irq_counter_info(&self) -> IrqCounterInfo {
        IrqCounterInfo {
            ticking_enabled: self.ticking_enabled,
            triggering_enabled: self.triggering_enabled,
            count: self.count,
        }
    }
}

#[derive(Clone, Copy)]
pub struct CounterBuilder {
    step: Option<NonZeroI8>,
    auto_triggered_by: Option<AutoTriggeredBy>,
    target_count: Option<u16>,
    trigger_on_forced_reload_with_target_count: bool,
    when_target_reached: Option<WhenTargetReached>,
    forced_reload_timing: Option<ForcedReloadTiming>,
    when_disabled_prevent: Option<WhenDisabledPrevent>,
    initial_reload_value: u16,
    initial_count: Option<u16>,
    prescaler: Prescaler,
}

impl CounterBuilder {
    pub const fn new() -> Self {
        Self {
            step: None,
            auto_triggered_by: None,
            target_count: None,
            trigger_on_forced_reload_with_target_count: false,
            when_target_reached: None,
            forced_reload_timing: None,
            when_disabled_prevent: None,
            initial_reload_value: 0,
            // Normally initial_reload_value is assigned to initial_count in build().
            initial_count: None,
            // A prescaler that doesn't actually delay anything.
            prescaler: Prescaler::DEFAULT,
        }
    }

    pub const fn step(&mut self, step: i8) -> &mut Self {
        self.step = Some(NonZeroI8::new(step).expect("step to not be zero"));
        self
    }

    pub const fn auto_triggered_by(&mut self, auto_triggered_by: AutoTriggeredBy, target_count: u16) -> &mut Self {
        self.auto_triggered_by = Some(auto_triggered_by);
        self.target_count = Some(target_count);
        self
    }

    pub const fn also_trigger_on_forced_reload_with_target_count(&mut self) -> &mut Self {
        self.trigger_on_forced_reload_with_target_count = true;
        self
    }

    pub const fn forced_reload_timing(&mut self, forced_reload_timing: ForcedReloadTiming) -> &mut Self {
        self.forced_reload_timing = Some(forced_reload_timing);
        self
    }

    pub const fn when_target_reached(&mut self, when_target_reached: WhenTargetReached) -> &mut Self {
        self.when_target_reached = Some(when_target_reached);
        self
    }

    pub const fn initial_count_and_reload_value(&mut self, value: u16) -> &mut Self {
        self.initial_count = Some(value);
        self.initial_reload_value = value;
        self
    }

    pub const fn initial_count(&mut self, value: u16) -> &mut Self {
        self.initial_count = Some(value);
        self
    }

    pub const fn initial_reload_value(&mut self, value: u16) -> &mut Self {
        self.initial_reload_value = value;
        self
    }

    pub const fn when_disabled_prevent(&mut self, when_disabled_prevent: WhenDisabledPrevent) -> &mut Self {
        self.when_disabled_prevent = Some(when_disabled_prevent);
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

    pub const fn build_directly_set_counter(self) -> DirectlySetCounter {
        assert!(self.forced_reload_timing.is_none());
        assert!(!self.trigger_on_forced_reload_with_target_count);
        DirectlySetCounter(self.build())
    }

    pub const fn build_reload_driven_counter(self) -> ReloadDrivenCounter {
        assert!(self.forced_reload_timing.is_some(),
            "forced_reload_timing must be set. If forced-reloading is not needed, use build_directly_settable() instead.");
        ReloadDrivenCounter {
            counter: self.build(),
            trigger_on_forced_reload_with_target_count: self.trigger_on_forced_reload_with_target_count,
            forced_reload_timing: self.forced_reload_timing.expect("forced_reload_timing to be set"),
            forced_reload_pending: false,
        }
    }

    const fn build(self) -> Counter {
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
            step: self.step.expect("step must be set"),
            auto_triggered_by: self.auto_triggered_by.expect("auto_triggered_by must be set"),
            target_count: self.target_count.expect("target must be set"),
            when_target_reached: self.when_target_reached.expect("when_target_reached must be set"),
            when_disabled_prevent,
            prescaler: self.prescaler,
            triggering_enabled: false,
            ticking_enabled,
            reload_value: self.initial_reload_value,
            count: self.initial_count.expect("initial_count must be set"),
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum AutoTriggeredBy {
    AlreadyOn,
    EndingOn,
    StepSizedTransitionTo,
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum WhenTargetReached {
    Stay,
    Reload,
    ContinueThenReloadAfter(u16),
}

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

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct TickResult {
    pub skipped: bool,
    pub wrapped: bool,
    pub triggered: bool,
}