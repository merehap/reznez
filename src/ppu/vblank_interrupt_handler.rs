use splitbits::replacebits;

use crate::memory::{cpu::cpu_pinout::CpuPinout, signal_level::{SignalLevel, nor}};

// Reference:
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

            int_ff: SignalLevel::Low,
        }
    }

    pub fn step_pclk_high(&mut self, cpu_pinout: &mut CpuPinout, input_signals: VBlankInterruptInputSignals) {
        self.vset_1 = input_signals.vset;

        // "Set" the SR latch
        if input_signals.vset.is_low() && self.vset_2.is_low() {
            let old_int_ff = self.int_ff;
            // SRlatch assumption: the SR reset input must not be High currently.
            // Set and Reset can't be simultaneously High because they are triggered on different scanlines.
            self.int_ff = SignalLevel::High;
            if old_int_ff != self.int_ff {
                log::info!("VIH PCLK int_ff went {:?}", self.int_ff);
            }

            // If the SR latch (int_ff) is put into the "invalid" state, the reset wins, because the "Q" output returns Low,
            // even though in normal circumstances a set would result in High.
            self.maybe_reset(cpu_pinout, input_signals);

            let old_nmi = cpu_pinout.nmi;
            // "INT" on the diagram. TODO: Const propagate.
            cpu_pinout.nmi = !nor(!input_signals.vbl, !self.int_ff);
            if old_nmi != cpu_pinout.nmi {
                log::info!("VIH PCLK NMI went {:?}", cpu_pinout.nmi);
            }
        }

        let read_enabled = input_signals.r2.is_low() && input_signals.dbe.is_low();
        if read_enabled {
            // "DB7_Out" on the diagram. Set the top bit of the CPU data bus.
            let v = self.db.is_low();
            cpu_pinout.data_bus = replacebits!(cpu_pinout.data_bus, "v... ....");
            log::info!("VIH PCLK data bus top bit set to {v}");
        } else {
            let old_db = self.db;
            self.db = !self.int_ff;
            if old_db != self.db {
                log::info!("VIH PCLK DB changed to {:?}", self.db);
            }
        }
    }

    pub fn step_pclk_low(&mut self) {
        self.vset_2 = !self.vset_1;
    }

    pub fn maybe_reset(&mut self, cpu_pinout: &mut CpuPinout, input_signals: VBlankInterruptInputSignals) {
        let read_enabled = input_signals.r2.is_low() && input_signals.dbe.is_low();
        if input_signals.vclr.is_high() || read_enabled {
            let old_int_ff = self.int_ff;
            // SRlatch assumption: the SR set input must not be High currently.
            // Set and Reset can't be simultaneously High because they are triggered on different scanlines.
            self.int_ff = SignalLevel::Low;
            if old_int_ff != self.int_ff {
                log::info!("VIH ASYNC int_ff went {:?}", self.int_ff);
            }
        }

        let old_nmi = cpu_pinout.nmi;
        // "INT" on the diagram. TODO: Const propagate.
        cpu_pinout.nmi = !nor(!input_signals.vbl, !self.int_ff);
        if old_nmi != cpu_pinout.nmi {
            log::info!("VIH ASYNC NMI went {:?}", cpu_pinout.nmi);
        }

        if read_enabled {
            // "DB7_Out" on the diagram. Set the top bit of the CPU data bus.
            let v = self.db.is_low();
            cpu_pinout.data_bus = replacebits!(cpu_pinout.data_bus, "v... ....");
            log::info!("VIH ASYNC data bus top bit set to {v}");
        } else {
            let old_db = self.db;
            self.db = !self.int_ff;
            if old_db != self.db {
                log::info!("VIH ASYNC DB changed to {:?}", self.db);
            }
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