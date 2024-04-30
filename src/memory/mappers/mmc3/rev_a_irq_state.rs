use crate::memory::mappers::mmc3::irq_state::IrqState;
use crate::memory::ppu::ppu_address::PpuAddress;
use crate::ppu::pattern_table::PatternTableSide;

pub struct RevAIrqState {
    pending: bool,
    enabled: bool,
    counter: u8,
    force_reload_counter: bool,
    counter_reload_value: u8,
    counter_suppression_cycles: u8,
    pattern_table_side: PatternTableSide,
}

impl RevAIrqState {
    pub fn new() -> Self {
        Self {
            pending: false,
            enabled: false,
            counter: 0,
            force_reload_counter: false,
            counter_reload_value: 0,
            counter_suppression_cycles: 0,
            pattern_table_side: PatternTableSide::Left,
        }
    }
}

impl IrqState for RevAIrqState {
    fn pending(&self) -> bool {
        self.pending
    }

    fn tick_counter(&mut self, address: PpuAddress) {
        if address.to_scroll_u16() >= 0x2000 {
            return;
        }

        let next_side = address.pattern_table_side();
        let should_tick_irq_counter =
            self.pattern_table_side == PatternTableSide::Left
            && next_side == PatternTableSide::Right
            && self.counter_suppression_cycles == 0;
        if next_side == PatternTableSide::Right {
            self.counter_suppression_cycles = 16;
        }

        if should_tick_irq_counter {
            let counter_started_positive = self.counter > 0;
            if self.counter == 0 || self.force_reload_counter {
                self.counter = self.counter_reload_value;
            } else {
                self.counter -= 1;
            }

            if self.enabled && self.counter == 0 && (counter_started_positive || self.force_reload_counter) {
                self.pending = true;
            }

            self.force_reload_counter = false;
        }

        self.pattern_table_side = next_side;
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

    fn disable(&mut self) {
        self.enabled = false;
        self.pending = false;
    }

    fn enable(&mut self) {
        self.enabled = true;
    }
}
