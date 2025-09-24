use crate::mapper::{DecrementingCounter, DecrementingCounterBuilder, ForcedReloadBehavior, AutoTriggeredBy};
use crate::mappers::mmc3::irq_state::IrqState;
use crate::memory::memory::Memory;
use crate::memory::ppu::ppu_address::PpuAddress;
use crate::ppu::pattern_table_side::PatternTableSide;
use crate::util::counter::WhenDisabledPrevent;
use crate::util::edge_detector::EdgeDetector;

const IRQ_COUNTER: DecrementingCounter = DecrementingCounterBuilder::new()
    .auto_trigger_on(AutoTriggeredBy::EndingOnZero)
    .auto_reload(true)
    .forced_reload_behavior(ForcedReloadBehavior::SetReloadValueOnNextTick)
    .when_disabled_prevent(WhenDisabledPrevent::Triggering)
    .build();

// Submapper 0
pub struct SharpIrqState {
    irq_counter: DecrementingCounter,
    counter_suppression_cycles: u8,
    pattern_table_transition_detector: EdgeDetector<PatternTableSide, { PatternTableSide::Right }>,
}

impl SharpIrqState {
    pub fn new() -> Self {
        Self {
            irq_counter: IRQ_COUNTER,
            counter_suppression_cycles: 0,
            pattern_table_transition_detector: EdgeDetector::new(),
        }
    }
}

impl IrqState for SharpIrqState {
    // Every time the PPU address changes.
    fn tick_counter(&mut self, mem: &mut Memory, address: PpuAddress) {
        if address.to_scroll_u16() >= 0x2000 {
            return;
        }

        let edge_detected = self.pattern_table_transition_detector.set_value_then_detect(address.pattern_table_side());
        let should_tick_irq_counter = edge_detected && self.counter_suppression_cycles == 0;
        if address.pattern_table_side() == PatternTableSide::Right {
            //println!("Resetting counter suppression cycles to 16");
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

    // Write 0xC000 (even addresses)
    fn set_counter_reload_value(&mut self, value: u8) {
        self.irq_counter.set_reload_value(value);
    }

    // Write 0xC001 (odd addresses)
    fn reload_counter(&mut self) {
        self.irq_counter.force_reload();
    }

    // Write 0xE000 (even addresses)
    fn disable(&mut self, mem: &mut Memory) {
        self.irq_counter.disable();
        mem.cpu_pinout.clear_mapper_irq_pending();
    }

    // Write 0xE001 (odd addresses)
    fn enable(&mut self) {
        self.irq_counter.enable();
    }
}
