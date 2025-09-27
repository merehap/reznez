use std::ops::RangeInclusive;

use crate::memory::memory::Memory;
use crate::memory::ppu::ppu_address::PpuAddress;
use crate::ppu::pattern_table_side::PatternTableSide;
use crate::util::counter::{AutoTriggeredBy, DecrementingCounter, DecrementingCounterBuilder, ForcedReloadBehavior, PrescalerBehaviorOnForcedReload, PrescalerTriggeredBy, WhenDisabledPrevent};
use crate::util::edge_detector::EdgeDetector;

pub struct IrqState {
    pub counter: DecrementingCounter,
    pub allowed_address_range: RangeInclusive<u16>,
    pub suppression_cycle_reload_value: u8,
    pub suppression_cycle_count: u8,
    pub pattern_table_side_detector: EdgeDetector<PatternTableSide>,
}

impl IrqState {
    pub const SHARP_IRQ_STATE: Self = Self::normal(SHARP_IRQ_COUNTER);
    pub const NEC_IRQ_STATE: Self = Self::normal(NEC_IRQ_COUNTER);
    pub const REV_A_IRQ_STATE: Self = Self::normal(REV_A_IRQ_COUNTER);
    pub const MC_ACC_IRQ_STATE: Self = Self {
        counter: MC_ACC_IRQ_COUNTER,
        allowed_address_range: 0..=0xFFFF,
        suppression_cycle_reload_value: 0,
        suppression_cycle_count: 0,
        pattern_table_side_detector: EdgeDetector::pattern_table_side_detector(PatternTableSide::Left),
    };

    const fn normal(counter: DecrementingCounter) -> Self {
        IrqState {
            counter,
            allowed_address_range: 0..=0x1FFF,
            suppression_cycle_reload_value: 16,
            suppression_cycle_count: 0,
            pattern_table_side_detector: EdgeDetector::pattern_table_side_detector(PatternTableSide::Right),
        }
    }

    // Note: There's no current cases that use both a prescaler and suppression cycles, so that combination is untested.
    pub fn tick_counter(&mut self, mem: &mut Memory, address: PpuAddress) {
        if !self.allowed_address_range.contains(&address.to_scroll_u16()) {
            return;
        }

        let not_suppressed = self.suppression_cycle_count == 0;
        let edge_detected = self.pattern_table_side_detector.set_value_then_detect(address.pattern_table_side());
        let should_tick_irq_counter = edge_detected && not_suppressed;

        if self.pattern_table_side_detector.matches_target(address.pattern_table_side()) {
            self.suppression_cycle_count = self.suppression_cycle_reload_value;
        }

        if should_tick_irq_counter {
            let triggered = self.counter.tick();
            if triggered {
                mem.cpu_pinout.set_mapper_irq_pending();
            }
        }
    }

    pub fn decrement_suppression_cycle_count(&mut self) {
        if self.suppression_cycle_count > 0 {
            self.suppression_cycle_count -= 1;
        }
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
        mem.cpu_pinout.clear_mapper_irq_pending();
    }

    // Write 0xE001 (odd addresses)
    pub fn enable(&mut self) {
        self.counter.enable();
    }
}

// ForcedReloadBehavior and WhenDisabledPrevent are the same for all MMC3 IRQs.

const SHARP_IRQ_COUNTER: DecrementingCounter = DecrementingCounterBuilder::new()
    .auto_triggered_by(AutoTriggeredBy::EndingOnZero)
    .auto_reload(true)

    .forced_reload_behavior(ForcedReloadBehavior::SetReloadValueOnNextTick)
    .when_disabled_prevent(WhenDisabledPrevent::Triggering)
    .build();

const MC_ACC_IRQ_COUNTER: DecrementingCounter = DecrementingCounterBuilder::new()
    .auto_triggered_by(AutoTriggeredBy::EndingOnZero)
    .auto_reload(true)
    .prescaler(8, PrescalerTriggeredBy::AlreadyZero, PrescalerBehaviorOnForcedReload::ClearCount)

    .forced_reload_behavior(ForcedReloadBehavior::SetReloadValueOnNextTick)
    .when_disabled_prevent(WhenDisabledPrevent::Triggering)
    .build();

const NEC_IRQ_COUNTER: DecrementingCounter = DecrementingCounterBuilder::new()
    .auto_triggered_by(AutoTriggeredBy::OneToZeroTransition)
    .auto_reload(true)

    .forced_reload_behavior(ForcedReloadBehavior::SetReloadValueOnNextTick)
    .when_disabled_prevent(WhenDisabledPrevent::Triggering)
    .build();

const REV_A_IRQ_COUNTER: DecrementingCounter = DecrementingCounterBuilder::new()
    .auto_triggered_by(AutoTriggeredBy::OneToZeroTransition)
    .also_trigger_on_forced_reload_of_zero()
    .auto_reload(true)

    .forced_reload_behavior(ForcedReloadBehavior::SetReloadValueOnNextTick)
    .when_disabled_prevent(WhenDisabledPrevent::Triggering)
    .build();