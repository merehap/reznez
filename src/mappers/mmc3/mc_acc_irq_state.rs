use crate::mappers::mmc3::irq_state::IrqState;
use crate::memory::memory::Memory;
use crate::memory::ppu::ppu_address::PpuAddress;
use crate::ppu::pattern_table_side::PatternTableSide;
use crate::util::edge_detector::EdgeDetector;

// Submapper 3
// TODO: Testing. No test ROM exists for this submapper.
pub struct McAccIrqState {
    enabled: bool,
    counter: u8,
    force_reload_counter: bool,
    counter_reload_value: u8,
    counter_suppression_cycles: u8,
    pattern_table_transition_detector: EdgeDetector<PatternTableSide, { PatternTableSide::Right }>,
}

impl McAccIrqState {
    pub fn new() -> Self {
        Self {
            enabled: false,
            counter: 0,
            force_reload_counter: false,
            counter_reload_value: 0,
            counter_suppression_cycles: 0,
            pattern_table_transition_detector: EdgeDetector::new(),
        }
    }
}

impl IrqState for McAccIrqState {
    fn tick_counter(&mut self, mem: &mut Memory, address: PpuAddress) {
        if address.to_scroll_u16() >= 0x2000 {
            return;
        }

        let edge_detected = self.pattern_table_transition_detector.set_value_then_detect(address.pattern_table_side());
        let should_tick_irq_counter = edge_detected && self.counter_suppression_cycles == 0;
        if address.pattern_table_side() == PatternTableSide::Left {
            self.counter_suppression_cycles = 16;
        }

        if should_tick_irq_counter {
            if self.counter == 0 || self.force_reload_counter {
                self.counter = self.counter_reload_value;
                self.force_reload_counter = false;
            } else {
                self.counter -= 1;
            }

            if self.enabled && self.counter == 0 {
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
        self.counter_reload_value = value;
    }

    fn reload_counter(&mut self) {
        self.counter = 0;
        self.force_reload_counter = true;
    }

    fn disable(&mut self, mem: &mut Memory) {
        self.enabled = false;
        mem.cpu_pinout.clear_mapper_irq_pending();
    }

    fn enable(&mut self) {
        self.enabled = true;
    }
}
