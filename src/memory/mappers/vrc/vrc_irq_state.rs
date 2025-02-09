use splitbits::splitbits_named;

use crate::memory::mapper::MapperParams;

pub struct VrcIrqState {
    enabled: bool,
    enable_upon_acknowledgement: bool,
    mode: IrqMode,
    counter_reload_low_value: u8,
    counter_reload_value: u8,
    counter: u8,
    // When prescaler drops below 0, the counter is incremented. Only relevant in scanline mode.
    prescaler: i16,
}

impl VrcIrqState {
    pub fn new() -> Self {
        Self {
            enabled: false,
            enable_upon_acknowledgement: false,
            mode: IrqMode::Scanline,
            counter_reload_low_value: 0,
            counter_reload_value: 0,
            counter: 0,
            prescaler: 341,
        }
    }

    pub fn step(&mut self, params: &mut MapperParams) {
        if !self.enabled {
            return;
        }

        if self.mode == IrqMode::Scanline {
            self.prescaler -= 3;
            if self.prescaler <= 0 {
                // Reset the prescaler.
                self.prescaler += 341;
            } else {
                // The prescaler hasn't reached zero yet, so the counter will not be incremented.
                return;
            }
        }

        if self.counter == 0xFF {
            params.set_irq_pending(true);
            self.counter = self.counter_reload_value;
        } else {
            self.counter += 1;
        }
    }

    pub fn set_reload_value(&mut self, value: u8) {
        self.counter_reload_value = value;
    }

    pub fn set_reload_value_low_bits(&mut self, value: u8) {
        self.counter_reload_low_value = value & 0b0000_1111;
    }

    pub fn set_reload_value_high_bits(&mut self, value: u8) {
        self.counter_reload_value = (value & 0b0000_1111) << 4 | self.counter_reload_low_value;
    }

    pub fn set_mode(&mut self, params: &mut MapperParams, value: u8) {
        params.set_irq_pending(false);

        let mode;
        (mode, self.enable_upon_acknowledgement, self.enabled) = splitbits_named!(value, ".....mae");
        self.mode = if mode { IrqMode::Cycle } else { IrqMode::Scanline };
        if self.enabled {
            self.counter = self.counter_reload_value;
        }
    }

    pub fn acknowledge(&mut self, params: &mut MapperParams) {
        params.set_irq_pending(false);
        if self.enable_upon_acknowledgement {
            self.enabled = true;
        }
    }
}

impl Default for VrcIrqState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(PartialEq, Debug)]
enum IrqMode {
    Scanline,
    Cycle,
}
