use crate::mapper::{DecrementingCounter, DecrementingCounterBuilder, ForcedReloadBehavior, TriggerOn};
use crate::mappers::mmc3::irq_state::IrqState;
use crate::memory::memory::Memory;
use crate::memory::ppu::ppu_address::PpuAddress;
use crate::ppu::pattern_table_side::PatternTableSide;
use crate::util::counter::WhenDisabled;
use crate::util::edge_detector::EdgeDetector;

const IRQ_COUNTER: DecrementingCounter = DecrementingCounterBuilder::new()
    .trigger_on(TriggerOn::EndingOnZero)
    .auto_reload(true)
    .forced_reload_behavior(ForcedReloadBehavior::OnNextTick)
    .when_disabled(WhenDisabled::PreventTriggering)
    .build();

// Submapper 3
// TODO: Testing. No test ROM exists for this submapper.
// FIXME: IRQ is still a bit off.
pub struct McAccIrqState {
    irq_counter: DecrementingCounter,
    prescaler: u8,
    pattern_table_transition_detector: EdgeDetector<PatternTableSide, { PatternTableSide::Left }>,
}

impl McAccIrqState {
    pub fn new() -> Self {
        Self {
            irq_counter: IRQ_COUNTER,
            prescaler: 0,
            pattern_table_transition_detector: EdgeDetector::new(),
        }
    }
}

impl IrqState for McAccIrqState {
    fn tick_counter(&mut self, mem: &mut Memory, address: PpuAddress) {
        let pattern_table_side_transitioned = self.pattern_table_transition_detector.set_value_then_detect(address.pattern_table_side());
        if !pattern_table_side_transitioned {
            return;
        }

        self.prescaler += 1;
        self.prescaler %= 8;

        if self.prescaler == 1 {
            let triggered = self.irq_counter.tick();
            if triggered {
                mem.cpu_pinout.set_mapper_irq_pending();
            }
        }
    }

    fn decrement_suppression_cycle_count(&mut self) {
        // Nothing to do here for MC-ACC
    }

    fn set_counter_reload_value(&mut self, value: u8) {
        self.irq_counter.set_reload_value(value);
    }

    fn reload_counter(&mut self) {
        self.prescaler = 0;
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
