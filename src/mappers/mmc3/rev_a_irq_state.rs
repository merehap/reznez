use crate::mapper::{DecrementingCounter, DecrementingCounterBuilder, ForcedReloadBehavior, TriggerOn};
use crate::mappers::mmc3::irq_state::IrqState;
use crate::memory::memory::Memory;
use crate::memory::ppu::ppu_address::PpuAddress;
use crate::ppu::pattern_table_side::PatternTableSide;
use crate::util::counter::WhenDisabled;
use crate::util::edge_detector::EdgeDetector;

const IRQ_COUNTER: DecrementingCounter = DecrementingCounterBuilder::new()
    .trigger_on(TriggerOn::OneToZeroTransition)
    .also_trigger_on_forced_reload_of_zero()
    .auto_reload(true)
    .forced_reload_behavior(ForcedReloadBehavior::OnNextTick)
    .when_disabled(WhenDisabled::PreventTriggering)
    .build();

// Submapper 1 (MMC6). Submapper 99 (MMC3). No submapper offically assigned for the MMC3 variant.
pub struct RevAIrqState {
    irq_counter: DecrementingCounter,
    counter_suppression_cycles: u8,
    pattern_table_transition_detector: EdgeDetector<PatternTableSide, { PatternTableSide::Right }>,
}

impl RevAIrqState {
    pub fn new() -> Self {
        Self {
            irq_counter: IRQ_COUNTER,
            counter_suppression_cycles: 0,
            pattern_table_transition_detector: EdgeDetector::new(),
        }
    }
}

impl IrqState for RevAIrqState {
    fn tick_counter(&mut self, mem: &mut Memory, address: PpuAddress) {
        if address.to_scroll_u16() >= 0x2000 {
            return;
        }

        let edge_detected = self.pattern_table_transition_detector.set_value_then_detect(address.pattern_table_side());
        let should_tick_irq_counter = edge_detected && self.counter_suppression_cycles == 0;
        if address.pattern_table_side() == PatternTableSide::Right {
            self.counter_suppression_cycles = 16;
        }

        if should_tick_irq_counter {
            let triggered = self.irq_counter.tick();
            if triggered {
                mem.cpu_pinout.set_mapper_irq_pending();
            }
        }
    }

    fn decrement_suppression_cycle_count(&mut self) {
        if self.counter_suppression_cycles > 0 {
            self.counter_suppression_cycles -= 1;
        }
    }

    fn set_counter_reload_value(&mut self, value: u8) {
        self.irq_counter.set_reload_value(value);
    }

    fn reload_counter(&mut self) {
        self.irq_counter.force_reload();
    }

    fn disable(&mut self, mem: &mut Memory) {
        self.irq_counter.disable();
        mem.cpu_pinout.clear_mapper_irq_pending();
    }

    fn enable(&mut self) {
        self.irq_counter.enable();
    }
}
