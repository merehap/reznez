use std::ops::RangeInclusive;

use crate::mapper::IrqCounterInfo;
use crate::memory::memory::Memory;
use crate::memory::ppu::ppu_address::PpuAddress;
use crate::ppu::pattern_table_side::PatternTableSide;
use crate::counter::counter::{AutoTriggerWhen, ReloadDrivenCounter, CounterBuilder, ForcedReloadTiming, PrescalerBehaviorOnForcedReload, PrescalerTriggeredBy, WhenDisabledPrevent};
use crate::util::edge_detector::EdgeDetector;

pub struct Mmc3IrqState {
    counter: ReloadDrivenCounter,
    allowed_address_range: RangeInclusive<u16>,
    suppressor: Suppressor,
    pattern_table_side_detector: EdgeDetector<PatternTableSide>,
}

impl Mmc3IrqState {
    // ForcedReloadBehavior and WhenDisabledPrevent are the same for all MMC3 IRQ varities.

    // The standard MMC3 IRQ behavior.
    pub const SHARP_IRQ_STATE: Self = Self {
        counter: CounterBuilder::new()
            .wraps(true)
            .full_range(0, 0xFFFF)
            .initial_range(0, 0)
            .step(-1)
            .auto_trigger_when(AutoTriggerWhen::EndingOn(0))
            .forced_reload_timing(ForcedReloadTiming::OnNextTick)
            .when_disabled_prevent(WhenDisabledPrevent::Triggering)
            .build_reload_driven_counter(),
        allowed_address_range: 0..=0x1FFF,
        suppressor: Suppressor::SUPPRESS_FOR_16_CYCLES,
        pattern_table_side_detector: EdgeDetector::pattern_table_side_detector(PatternTableSide::Right),
    };
    // Same as Sharp except that automatic IRQs are ONLY triggered on a 1 to 0 transition of the count, not when it was already 0.
    pub const NEC_IRQ_STATE: Self = Self {
        counter: CounterBuilder::new()
            .step(-1)
            .wraps(true)
            .full_range(0, 0xFFFF)
            .initial_range(0, 0)
            .auto_trigger_when(AutoTriggerWhen::StepSizedTransitionTo(0))
            .forced_reload_timing(ForcedReloadTiming::OnNextTick)
            .when_disabled_prevent(WhenDisabledPrevent::Triggering)
            .build_reload_driven_counter(),
        allowed_address_range: 0..=0x1FFF,
        suppressor: Suppressor::SUPPRESS_FOR_16_CYCLES,
        pattern_table_side_detector: EdgeDetector::pattern_table_side_detector(PatternTableSide::Right),
    };
    // Same as NEC except that forcing a reload of 0 will also trigger an IRQ.
    pub const REV_A_IRQ_STATE: Self = Self {
        counter: CounterBuilder::new()
            .step(-1)
            .wraps(true)
            .full_range(0, 0xFFFF)
            .initial_range(0, 0)
            .auto_trigger_when(AutoTriggerWhen::StepSizedTransitionTo(0))
            .also_trigger_on_forced_reload_with_target_count()
            .forced_reload_timing(ForcedReloadTiming::OnNextTick)
            .when_disabled_prevent(WhenDisabledPrevent::Triggering)
            .build_reload_driven_counter(),
        allowed_address_range: 0..=0x1FFF,
        suppressor: Suppressor::SUPPRESS_FOR_16_CYCLES,
        pattern_table_side_detector: EdgeDetector::pattern_table_side_detector(PatternTableSide::Right),
    };
    // Very different from the other MMC3 IRQs since it has a prescaler, doesn't filter PPU addresses,
    // triggers on pattern table side transitions to the LEFT not the right, and doesn't suppress repeats on transition.
    pub const MC_ACC_IRQ_STATE: Self = Self {
        counter: CounterBuilder::new()
            .step(-1)
            .wraps(true)
            .full_range(0, 0xFFFF)
            .initial_range(0, 0)
            .auto_trigger_when(AutoTriggerWhen::EndingOn(0))
            .forced_reload_timing(ForcedReloadTiming::OnNextTick)
            .when_disabled_prevent(WhenDisabledPrevent::Triggering)
            .prescaler(8, PrescalerTriggeredBy::AlreadyZero, PrescalerBehaviorOnForcedReload::ClearCount)
            .build_reload_driven_counter(),
        allowed_address_range: 0..=0xFFFF,
        suppressor: Suppressor::NEVER_SUPPRESS,
        pattern_table_side_detector: EdgeDetector::pattern_table_side_detector(PatternTableSide::Left),
    };

    pub fn tick_counter(&mut self, mem: &mut Memory, address: PpuAddress) {
        if !self.allowed_address_range.contains(&address.to_u16()) {
            return;
        }

        let switched_to_target_side = self.pattern_table_side_detector.set_value_then_detect(address.pattern_table_side());
        let should_tick_irq_counter = switched_to_target_side && !self.suppressor.suppressed();

        // Keep re-suppressing ticks for as long as we are on the target pattern table side.
        // If NEVER_SUPPRESS is specified, this does nothing.
        if self.pattern_table_side_detector.matches_target(address.pattern_table_side()) {
            self.suppressor.suppress();
        }

        if should_tick_irq_counter && self.counter.tick().triggered {
            mem.cpu_pinout.assert_mapper_irq();
        }
    }

    pub fn decrement_suppression_cycle_count(&mut self) {
        self.suppressor.decrement();
    }

    // Write 0xC000 (even addresses)
    pub fn set_counter_reload_value(&mut self, value: u8) {
        self.counter.set_reload_value(value);
    }

    // Write 0xC001 (odd addresses)
    pub fn reload_counter(&mut self) {
        self.counter.force_reload();
    }

    // Write 0xE000 (even addresses)
    pub fn disable(&mut self, mem: &mut Memory) {
        self.counter.disable();
        mem.cpu_pinout.acknowledge_mapper_irq();
    }

    // Write 0xE001 (odd addresses)
    pub fn enable(&mut self) {
        self.counter.enable();
    }

    pub fn irq_counter_info(&self) -> IrqCounterInfo {
        self.counter.to_irq_counter_info()
    }
}

struct Suppressor {
    reload_value: u8,
    cycles_remaining: u8,
}

impl Suppressor {
    const NEVER_SUPPRESS: Self = Self {
        reload_value: 0,
        cycles_remaining: 0,
    };
    const SUPPRESS_FOR_16_CYCLES: Self = Self {
        reload_value: 16,
        cycles_remaining: 0,
    };

    fn suppressed(&self) -> bool {
        self.cycles_remaining > 0
    }

    fn suppress(&mut self) {
        self.cycles_remaining = self.reload_value;
    }

    fn decrement(&mut self) {
        self.cycles_remaining = self.cycles_remaining.saturating_sub(1);
    }
}