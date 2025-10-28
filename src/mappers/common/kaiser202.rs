use crate::counter::counter::{ReloadDrivenCounter, CounterBuilder, AutoTriggerWhen, ForcedReloadTiming, WhenDisabledPrevent};

pub const IRQ_COUNTER: ReloadDrivenCounter = CounterBuilder::new()
    .step(1)
    .wraps(true)
    .full_range(0, 0xFFFF)
    .initial_count(0)
    .auto_trigger_when(AutoTriggerWhen::Wrapping)
    // TODO: Verify.
    .forced_reload_timing(ForcedReloadTiming::Immediate)
    .when_disabled_prevent(WhenDisabledPrevent::CountingAndTriggering)
    .build_reload_driven_counter();