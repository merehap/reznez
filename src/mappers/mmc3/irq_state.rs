use std::ops::RangeInclusive;

use crate::memory::memory::Memory;
use crate::memory::ppu::ppu_address::PpuAddress;
use crate::ppu::pattern_table_side::PatternTableSide;
use crate::util::counter::{DecrementingCounter, DecrementingCounterBuilder, AutoTriggeredBy, ForcedReloadBehavior, WhenDisabledPrevent};
use crate::util::edge_detector::EdgeDetector;

pub struct IrqState {
    pub counter: DecrementingCounter,
    pub allowed_address_range: RangeInclusive<u16>,
    pub suppression_cycle_reload_value: u8,
    pub suppression_cycle_count: u8,
    pub pattern_table_side_detector: EdgeDetector<PatternTableSide>,
    pub prescaler_multiple: u8,
    pub prescaler: u8,
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
        prescaler_multiple: 8,
        prescaler: 0,
    };

    const fn normal(counter: DecrementingCounter) -> Self {
        IrqState {
            counter,
            allowed_address_range: 0..=0x1FFF,
            suppression_cycle_reload_value: 16,
            suppression_cycle_count: 0,
            pattern_table_side_detector: EdgeDetector::pattern_table_side_detector(PatternTableSide::Right),
            prescaler_multiple: 1,
            prescaler: 0,
        }
    }

    // Note: There's no current cases that use both a prescaler and suppression cycles, so that combination is untested.
    pub fn tick_counter(&mut self, mem: &mut Memory, address: PpuAddress) {
        if !self.allowed_address_range.contains(&address.to_scroll_u16()) {
            return;
        }

        let not_suppressed = self.suppression_cycle_count == 0;
        let edge_detected = self.pattern_table_side_detector.set_value_then_detect(address.pattern_table_side());
        let should_tick_irq_counter = edge_detected && not_suppressed && self.prescaler == 0;

        if edge_detected {
            self.prescaler += 1;
            self.prescaler %= self.prescaler_multiple;
        }

        if address.pattern_table_side() == self.pattern_table_side_detector.target_value() {
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
        self.prescaler = 0;
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

const SHARP_IRQ_COUNTER: DecrementingCounter = IRQ_COUNTER_BUILDER
    .auto_trigger_on(AutoTriggeredBy::EndingOnZero)
    .build();

// TODO: Verify that the MC-ACC counter actually is the same as SHARP.
// Alternately, move prescaler logic into DecrementingCounter.
const MC_ACC_IRQ_COUNTER: DecrementingCounter = IRQ_COUNTER_BUILDER
    .auto_trigger_on(AutoTriggeredBy::EndingOnZero)
    .build();

const NEC_IRQ_COUNTER: DecrementingCounter = IRQ_COUNTER_BUILDER
    .auto_trigger_on(AutoTriggeredBy::OneToZeroTransition)
    .build();

const REV_A_IRQ_COUNTER: DecrementingCounter = IRQ_COUNTER_BUILDER
    .auto_trigger_on(AutoTriggeredBy::OneToZeroTransition)
    .also_trigger_on_forced_reload_of_zero()
    .build();

const IRQ_COUNTER_BUILDER: DecrementingCounterBuilder = *DecrementingCounterBuilder::new()
    .auto_reload(true)
    .forced_reload_behavior(ForcedReloadBehavior::SetReloadValueOnNextTick)
    .when_disabled_prevent(WhenDisabledPrevent::Triggering);