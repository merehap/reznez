use splitbits::replacebits;

use crate::memory::{cpu::cpu_pinout::CpuPinout, signal_level::{SignalLevel, nor}};

// Reference:
// TODO: Support DB7_Out?
pub struct VBlankInterruptHandler {
    // D latches
    vset_1: SignalLevel,
    vset_2: SignalLevel,
    db: SignalLevel,

    // SR latch
    int_ff: SignalLevel,
}

impl VBlankInterruptHandler {
    pub fn new() -> Self {
        Self {
            vset_1: SignalLevel::High,
            vset_2: SignalLevel::Low,
            db: SignalLevel::High,

            int_ff: SignalLevel::High,
        }
    }

    pub fn step_pclk_high(&mut self, cpu_pinout: &mut CpuPinout, input_signals: VBlankInterruptInputSignals) {
        self.vset_1 = input_signals.vset;

        // "Set" the SR latch
        if input_signals.vset.is_low() && self.vset_2.is_low() {
            // SRlatch assumption: the SR reset input must not be High currently.
            // Set and Reset can't be simultaneously High because they are triggered on different scanlines.
            self.int_ff = SignalLevel::High;

            // "INT" on the diagram. TODO: Const propagate.
            cpu_pinout.nmi = nor(!input_signals.vbl, self.int_ff);
        }

        let read_enabled = input_signals.r2.is_low() && input_signals.dbe.is_low();
        if read_enabled {
            // "DB7_Out" on the diagram. Set the top bit of the CPU data bus.
            let v = self.db.is_high();
            cpu_pinout.data_bus = replacebits!(cpu_pinout.data_bus, "v... ....");
        } else {
            self.db = self.int_ff;
        }
    }

    pub fn step_pclk_low(&mut self) {
        self.vset_2 = !self.vset_1;
    }

    pub fn maybe_clear(&mut self, cpu_pinout: &mut CpuPinout, input_signals: VBlankInterruptInputSignals) {
        let read_enabled = input_signals.r2.is_low() && input_signals.dbe.is_low();
        if input_signals.vclr.is_high() || read_enabled {
            // SRlatch assumption: the SR set input must not be High currently.
            // Set and Reset can't be simultaneously High because they are triggered on different scanlines.
            self.int_ff = SignalLevel::Low;

            // "INT" on the diagram. TODO: Const propagate.
            cpu_pinout.nmi = nor(!input_signals.vbl, self.int_ff);

            // "DB7_Out" on the diagram. Set the top bit of the CPU data bus.
            let v = self.db.is_high();
            cpu_pinout.data_bus = replacebits!(cpu_pinout.data_bus, "v... ....");
        } else {
            self.db = self.int_ff;
        }
    }
}

#[derive(Clone, Copy)]
pub struct VBlankInterruptInputSignals {
    pub vset: SignalLevel,
    pub vbl: SignalLevel,
    pub vclr: SignalLevel,
    pub r2: SignalLevel,
    pub dbe: SignalLevel,
}