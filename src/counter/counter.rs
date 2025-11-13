use std::num::{NonZeroI8, NonZeroU8};

use ux::u4;

use crate::counter::irq_counter_info::IrqCounterInfo;
pub use crate::counter::when_disabled_prevent::WhenDisabledPrevent;

// A counter where the count can be set directly, and can't be force-reloaded.
pub struct DirectlySetCounter(Counter);

impl DirectlySetCounter {
    pub fn enable(&mut self) { self.0.enable(); }
    pub fn disable(&mut self) { self.0.disable(); }
    pub fn set_enabled(&mut self, enabled: bool) { self.0.set_enabled(enabled); }
    pub fn tick(&mut self) -> TickResult { self.0.tick(false, false) }
    pub fn to_irq_counter_info(&self) -> IrqCounterInfo { self.0.to_irq_counter_info() }

    pub fn count_low_byte(&self) -> u8 {
        self.0.count.to_be_bytes()[1]
    }

    pub fn count_high_byte(&self) -> u8 {
        self.0.count.to_be_bytes()[0]
    }

    pub fn set_count(&mut self, count: u8) {
        self.0.count = count.into();
    }

    pub fn set_count_low_byte(&mut self, value: u8) {
        self.0.count = (self.0.count & 0xFF00) | u16::from(value);
    }

    pub fn set_count_high_byte(&mut self, value: u8) {
        self.0.count = (self.0.count & 0x00FF) | (u16::from(value) << 8);
    }

    pub fn set_step(&mut self, step: NonZeroI8) {
        self.0.step = step;
    }

    pub fn set_prescaler_count(&mut self, count: u8) {
        self.0.prescaler.count = count;
    }

    pub fn set_prescaler_mask(&mut self, mask: u8) {
        self.0.prescaler.mask = mask;
    }

    pub fn set_prescaler_step(&mut self, step: NonZeroI8) {
        self.0.prescaler.step = step;
    }

    // The vast majority of use-cases should just call enable/disable instead of this.
    pub fn set_counting_enabled(&mut self, counting_enabled: bool) {
        self.0.counting_enabled = counting_enabled;
    }

    // The vast majority of use-cases should just call enable/disable instead of this.
    pub fn enable_counting(&mut self) {
        self.0.counting_enabled = true;
    }

    // The vast majority of use-cases should just call enable/disable instead of this.
    pub fn disable_counting(&mut self) {
        self.0.counting_enabled = false;
    }

    // The vast majority of use-cases should just call enable/disable instead of this.
    pub fn set_triggering_enabled(&mut self, triggering_enabled: bool) {
        self.0.triggering_enabled = triggering_enabled;
    }

    // The vast majority of use-cases should just call enable/disable instead of this.
    pub fn enable_triggering(&mut self) {
        self.0.triggering_enabled = true;
    }

    // The vast majority of use-cases should just call enable/disable instead of this.
    pub fn disable_triggering(&mut self) {
        self.0.triggering_enabled = false;
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
    pub fn set_enabled(&mut self, enabled: bool) { self.counter.set_enabled(enabled); }
    pub fn to_irq_counter_info(&self) -> IrqCounterInfo { self.counter.to_irq_counter_info() }

    pub fn set_reload_value(&mut self, value: u8) {
        self.counter.modify_reload_value(|_| u16::from(value));
    }

    pub fn set_reload_value_low_byte(&mut self, low_byte: u8) {
        self.counter.modify_reload_value(|reload_value| (reload_value & 0xFF00) | u16::from(low_byte));
    }

    pub fn set_reload_value_high_byte(&mut self, high_byte: u8) {
        self.counter.modify_reload_value(|reload_value| (reload_value & 0x00FF) | (u16::from(high_byte) << 8));
    }

    pub fn set_reload_value_lowest_nybble(&mut self, nibble: u4) {
        self.counter.modify_reload_value(|reload_value| (reload_value & 0xFFF0) | u16::from(nibble));
    }

    pub fn set_reload_value_second_lowest_nybble(&mut self, nibble: u4) {
        self.counter.modify_reload_value(|reload_value| (reload_value & 0xFF0F) | (u16::from(nibble) << 4));
    }

    pub fn set_reload_value_second_highest_nybble(&mut self, nibble: u4) {
        self.counter.modify_reload_value(|reload_value| (reload_value & 0xF0FF) | (u16::from(nibble) << 8));
    }

    pub fn set_reload_value_highest_nybble(&mut self, nibble: u4) {
        self.counter.modify_reload_value(|reload_value| (reload_value & 0x0FFF) | (u16::from(nibble) << 12));
    }

    pub fn force_reload(&mut self) {
        if self.counter.prescaler.behavior_on_forced_reload == PrescalerBehaviorOnForcedReload::ClearCount {
            self.counter.prescaler.count = 0;
        }

        match self.forced_reload_timing {
            ForcedReloadTiming::Immediate => self.counter.count = self.counter.reload_value(),
            ForcedReloadTiming::OnNextTick => self.forced_reload_pending = true,
        }
    }

    pub fn tick(&mut self) -> TickResult {
        // TODO: Determine if a forced reload needs to clear the counter before the reloading actually occurs for some cases.
        // Some documentation claims this. This would only be relevant for AlreadyZero behavior since it
        // affects whether the counter is triggered or not during a forced reload.
        let triggered_by_forcing = self.trigger_on_forced_reload_with_target_count
            && self.forced_reload_pending
            && self.counter.reload_value() == self.counter.target_count();

        let result = self.counter.tick(self.forced_reload_pending, triggered_by_forcing);
        if !result.skipped {
            self.forced_reload_pending = false;
        }

        result
    }
}

struct Counter {
    full_range: Range,
    current_range: Range,
    wraps: bool,

    // Immutable settings determined at compile time
    step: NonZeroI8,
    auto_triggered_by: AutoTriggerWhen,
    when_disabled_prevent: WhenDisabledPrevent,

    // State
    triggering_enabled: bool,
    counting_enabled: bool,
    count: u16,
    prescaler: Prescaler,
}

impl Counter {
    fn enable(&mut self) {
        self.triggering_enabled = true;
        self.counting_enabled = true;
    }

    fn disable(&mut self) {
        match self.when_disabled_prevent {
            WhenDisabledPrevent::Counting => self.counting_enabled = false,
            WhenDisabledPrevent::Triggering => self.triggering_enabled = false,
            WhenDisabledPrevent::CountingAndTriggering => {
                self.counting_enabled = false;
                self.triggering_enabled = false;
            }
        }
    }

    fn set_enabled(&mut self, enabled: bool) {
        if enabled {
            self.enable();
        } else {
            self.disable();
        }
    }

    fn target_count(&self) -> u16 {
        match self.auto_triggered_by {
            AutoTriggerWhen::Wrapping => self.reload_value(),
            AutoTriggerWhen::EndingOn(end_count) => end_count,
            AutoTriggerWhen::StepSizedTransitionTo(end_count) => end_count,
        }
    }

    fn end_count(&self) -> u16 {
        if self.step.is_positive() {
            self.current_range.max
        } else {
            self.current_range.min
        }
    }

    fn reload_value(&self) -> u16 {
        if self.step.is_positive() {
            self.current_range.min
        } else {
            self.current_range.max
        }
    }

    fn modify_reload_value<M>(&mut self, modify: M)
    where M: Fn(u16) -> u16 {
        if self.step.is_positive() {
            self.current_range = Range::new(modify(self.current_range.min), self.current_range.max);
        } else {
            self.current_range = Range::new(self.current_range.min, modify(self.current_range.max));
        }

        assert!(self.current_range.is_subrange_of(self.full_range));
    }

    fn tick(&mut self, forced_reload_pending: bool, triggered_by_forcing: bool) -> TickResult {
        let old_count = self.count;
        let mut wrapped = false;
        if self.counting_enabled {
            // ASSUMPTION: Forced reloads and triggers (auto and forced) are prescaler-delayed, not just actual counter ticks.
            // NOTE: It's not clear how a counter that can only disable counting can support a prescaler, so that fails compile.
            let prescaler_triggered = self.prescaler.tick();
            if !prescaler_triggered {
                // The prescaler didn't trigger yet, so don't tick nor trigger the actual counter.
                return TickResult { skipped: true, wrapped: false, triggered: false };
            }
            
            let end_count_reached = old_count == self.end_count();
            if forced_reload_pending {
                self.count = self.reload_value();
            } else if !end_count_reached {
                self.count = old_count.saturating_add_signed(self.step.get() as i16);
                if old_count != self.target_count() && self.count != self.target_count() {
                    assert_eq!(self.count > self.target_count(), old_count > self.target_count(), "Stepped OVER the target count.");
                }
            } else if self.wraps {
                self.count = self.reload_value();
                wrapped = true;
            } else {
                // Stay on the end count, leaving the count unchanged.
            }
        }

        let new_count = self.count;
        // The triggering behavior is fixed at compile time, so the same branch will be taken every time here.
        let auto_triggered = match self.auto_triggered_by {
            AutoTriggerWhen::Wrapping => wrapped,
            AutoTriggerWhen::EndingOn(_) => new_count == self.target_count(),
            AutoTriggerWhen::StepSizedTransitionTo(_) =>
                i32::from(self.target_count()) - i32::from(old_count) == i32::from(self.step.get())
                    && new_count == self.target_count(),
        };

        let trigger_if_enabled = auto_triggered || triggered_by_forcing;
        let triggered = trigger_if_enabled && self.triggering_enabled;

        TickResult { skipped: false, wrapped, triggered }
    }

    fn to_irq_counter_info(&self) -> IrqCounterInfo {
        IrqCounterInfo {
            counting_enabled: self.counting_enabled,
            triggering_enabled: self.triggering_enabled,
            count: self.count,
        }
    }
}

#[derive(Clone, Copy)]
pub struct CounterBuilder {
    full_range: Option<Range>,
    initial_range: Option<Range>,
    initial_count: Option<u16>,
    wraps: Option<bool>,

    step: Option<NonZeroI8>,
    auto_triggered_by: Option<AutoTriggerWhen>,
    trigger_on_forced_reload_with_target_count: bool,
    forced_reload_timing: Option<ForcedReloadTiming>,
    when_disabled_prevent: Option<WhenDisabledPrevent>,
    prescaler: Prescaler,
}

impl CounterBuilder {
    pub const fn new() -> Self {
        Self {
            full_range: None,
            initial_range: None,
            initial_count: None,
            wraps: None,

            step: None,
            auto_triggered_by: None,
            trigger_on_forced_reload_with_target_count: false,
            forced_reload_timing: None,
            when_disabled_prevent: None,
            // A prescaler that doesn't actually delay anything.
            prescaler: Prescaler::DEFAULT,
        }
    }

    pub const fn full_range(&mut self, start: u16, end: u16) -> &mut Self {
        self.full_range = Some(Range::new(start, end));
        self
    }

    pub const fn initial_range(&mut self, start: u16, end: u16) -> &mut Self {
        self.initial_range = Some(Range::new(start, end));
        self
    }

    pub const fn initial_count(&mut self, initial_count: u16) -> &mut Self {
        self.initial_count = Some(initial_count);
        self
    }

    pub const fn wraps(&mut self, wraps: bool) -> &mut Self {
        self.wraps = Some(wraps);
        self
    }

    pub const fn step(&mut self, step: i8) -> &mut Self {
        self.step = Some(NonZeroI8::new(step).expect("step to not be zero"));
        self
    }

    pub const fn auto_trigger_when(&mut self, auto_triggered_when: AutoTriggerWhen) -> &mut Self {
        self.auto_triggered_by = Some(auto_triggered_when);
        self
    }

    // TODO: This should probably be eliminated. Maybe the caller can implement this instead.
    pub const fn also_trigger_on_forced_reload_with_target_count(&mut self) -> &mut Self {
        self.trigger_on_forced_reload_with_target_count = true;
        self
    }

    pub const fn forced_reload_timing(&mut self, forced_reload_timing: ForcedReloadTiming) -> &mut Self {
        self.forced_reload_timing = Some(forced_reload_timing);
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
            step: NonZeroI8::new(1).unwrap(),
            mask: 0xFF,

        };
        self
    }

    pub const fn build_directly_set_counter(self) -> DirectlySetCounter {
        assert!(self.forced_reload_timing.is_none());
        assert!(!self.trigger_on_forced_reload_with_target_count);
        assert!(self.initial_range.is_none(), "DirectlySetCounters must only use full_range: initial_range must not be set.");
        DirectlySetCounter(self.build())
    }

    pub const fn build_reload_driven_counter(self) -> ReloadDrivenCounter {
        assert!(self.forced_reload_timing.is_some(),
            "forced_reload_timing must be set. If forced-reloading is not needed, use build_directly_settable() instead.");
        let counter = self.build();
        assert!(!self.trigger_on_forced_reload_with_target_count || !matches!(counter.auto_triggered_by, AutoTriggerWhen::Wrapping));
        ReloadDrivenCounter {
            counter,
            trigger_on_forced_reload_with_target_count: self.trigger_on_forced_reload_with_target_count,
            forced_reload_timing: self.forced_reload_timing.expect("forced_reload_timing to be set"),
            forced_reload_pending: false,
        }
    }

    const fn build(mut self) -> Counter {
        let when_disabled_prevent = self.when_disabled_prevent.expect("when_disabled must be set");
        let counting_enabled = match when_disabled_prevent {
            // Counters that CANNOT disable counting will always have counting enabled.
            WhenDisabledPrevent::Triggering => true,
            // Counters that CAN disable counting should START with counting disabled.
            WhenDisabledPrevent::Counting | WhenDisabledPrevent::CountingAndTriggering => false,
        };

        let safe_for_prescaler = !matches!(when_disabled_prevent, WhenDisabledPrevent::Counting);
        assert!(safe_for_prescaler || !self.prescaler.enabled(),
            "WhenDisabledPrevent::Counting must not be specified at the same as a prescaler.");

        let wraps = self.wraps.expect("wraps must be set");
        let auto_triggered_by =  self.auto_triggered_by.expect("auto_triggered_by must be set");
        if matches!(auto_triggered_by, AutoTriggerWhen::Wrapping) {
            assert!(wraps, "Enable wrapping in order to AutoTriggerWhen::Wrapping.");
        }

        let max_range = self.full_range.expect("max_range must be set");
        let current_range = self.initial_range.unwrap_or(max_range);
        if self.initial_count.is_none() && current_range.min == current_range.max {
            // We've verified that there is only one value the initial_count could be, so don't force the caller to specify it.
            self.initial_count = Some(current_range.min);
        }

        let count = self.initial_count.expect("initial_count must be set");
        assert!(current_range.contains(count), "Initial count must be within initial_range (and full_range)");

        Counter {
            full_range: max_range,
            current_range,
            wraps,
            step: self.step.expect("step must be set"),
            auto_triggered_by,
            when_disabled_prevent,
            prescaler: self.prescaler,
            triggering_enabled: false,
            counting_enabled,
            count,
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum AutoTriggerWhen {
    Wrapping,
    EndingOn(u16),
    StepSizedTransitionTo(u16),
}

/*
#[derive(PartialEq, Eq, Clone, Copy)]
pub enum WhenTargetReached {
    Stay,
    Reload,
    ContinueThenReloadAfter(u16),
}
    */

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum ForcedReloadTiming {
    Immediate,
    OnNextTick,
}

#[derive(Clone, Copy, Debug)]
pub struct Prescaler {
    // Immutable settings determined at compile time
    multiple: NonZeroU8,
    triggered_by: PrescalerTriggeredBy,
    behavior_on_forced_reload: PrescalerBehaviorOnForcedReload,

    // State
    count: u8,
    mask: u8,
    step: NonZeroI8,
}

impl Prescaler {
    // A multiple of 1 effectively means the prescaler has no effect, regardless of which triggered_by behavior is used.
    const DEFAULT: Self = Self {
        multiple: NonZeroU8::new(1).unwrap(),
        triggered_by: PrescalerTriggeredBy::AlreadyZero,
        behavior_on_forced_reload: PrescalerBehaviorOnForcedReload::DoNothing,
        count: 0,
        mask: 0xFF,
        step: NonZeroI8::new(1).unwrap(),
    };

    fn tick(&mut self) -> bool {
        let old_count = self.count;
        self.count = self.count.wrapping_add_signed(self.step.get());
        self.count %= self.multiple;
        let new_count = self.count;

        match self.triggered_by {
            PrescalerTriggeredBy::AlreadyZero => old_count & self.mask == 0,
            PrescalerTriggeredBy::WrappingToZero => new_count & self.mask == 0,
        }
    }

    const fn enabled(self) -> bool {
        self.multiple.get() > 1
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

#[derive(Clone, Copy)]
pub struct Range {
    min: u16,
    max: u16,
}

impl Range {
    const fn new(start: u16, end: u16) -> Self {
        assert!(start <= end);
        Range { min: start, max: end }
    }

    const fn is_subrange_of(self, other: Range) -> bool {
        self.min >= other.min && self.max <= other.max
    }

    const fn contains(self, value: u16) -> bool {
        self.min <= value && value <= self.max
    }
}