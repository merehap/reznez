use crate::memory::ppu::ppu_address::PpuAddress;
use crate::ppu::pattern_table::PatternTableSide;

pub struct NecIrqState {
    pending: bool,
    enabled: bool,
    counter: u8,
    force_reload_counter: bool,
    counter_reload_value: u8,
    counter_suppression_cycles: u8,
    pattern_table_side: PatternTableSide,
}

impl NecIrqState {
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

    pub fn pending(&self) -> bool {
        self.pending
    }

    pub fn tick_counter(&mut self, address: PpuAddress) {
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
            let old_counter_value = self.counter;
            if self.counter == 0 || self.force_reload_counter {
                self.counter = self.counter_reload_value;
                self.force_reload_counter = false;
            } else {
                self.counter -= 1;
            }

            // NEC triggers an IRQ when the counter transitions from 1 to 0,
            // whether from decrement or forced reload.
            if self.enabled && self.counter == 0 && old_counter_value == 1 {
                self.pending = true;
            }
        }

        self.pattern_table_side = next_side;
    }

    pub fn decrement_suppression_cycle_count(&mut self) {
        if self.counter_suppression_cycles > 0 {
            self.counter_suppression_cycles -= 1;
        }
    }

    pub fn set_counter_reload_value(&mut self, value: u8) {
        self.counter_reload_value = value;
    }

    pub fn reload_counter(&mut self) {
        self.counter = 0;
        self.force_reload_counter = true;
    }

    pub fn disable(&mut self) {
        self.enabled = false;
        self.pending = false;
    }

    pub fn enable(&mut self) {
        self.enabled = true;
    }
}