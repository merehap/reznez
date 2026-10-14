use splitbits::replacebits;

use crate::memory::cpu::cpu_pinout::CpuPinout;
use crate::memory::signal_level::{SignalLevel, nor, nor3};

// Reference: https://github.com/emu-russia/breaks/blob/master/BreakingNESWiki_DeepL/PPU/fsm.md#vblank-interrupt-handling
// FIXME: VCLR actually triggers off PCLK high

/*
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
        }

        let read_enabled = input_signals.r2.is_low() && input_signals.dbe.is_low();
        if read_enabled {
            // If the SR latch (int_ff) is put into the "invalid" state, the reset wins, because the "Q" output returns Low,
            // even though in normal circumstances a set would result in High.
            self.maybe_reset(cpu_pinout, input_signals);

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

        let old_nmi = cpu_pinout.nmi;
        // "INT" on the diagram. TODO: Const propagate.
        cpu_pinout.nmi = !nor(!input_signals.vbl, !self.int_ff);
        if old_nmi != cpu_pinout.nmi {
            log::info!("VIH PCLK NMI went {:?}", cpu_pinout.nmi);
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
*/

pub struct VBlankInterruptHandler {
    vset_1: DLatch,
    vset_2: DLatch,
    db: DLatch,
    int_ff: SRLatch,
}

impl VBlankInterruptHandler {
    pub fn new() -> Self {
        Self {
            vset_1: DLatch::new(SignalLevel::High),
            vset_2: DLatch::new(SignalLevel::Low),
            db: DLatch::new(SignalLevel::High),
            int_ff: SRLatch::new(SignalLevel::Low),
        }
    }

    pub fn update(&mut self, cpu_pinout: &mut CpuPinout, VbihInputs { pclk, vset, vbl, vclr, r2, dbe }: VbihInputs) {
        self.vset_1.update(DLInputs {
            e: pclk,
            d: vset,
        });
        self.vset_2.update(DLInputs {
            e: !pclk,
            d: self.vset_1.nq(),
        });
        let set = nor3(!pclk, vset, self.vset_2.q());

        let read_enabled = nor(r2, dbe);
        let reset = vclr | read_enabled;

        self.int_ff.update(SRInputs {
            s: set,
            r: reset,
        });

        let int = nor(!vbl, !self.int_ff.q());
        cpu_pinout.nmi = !int;

        let old_db_q = self.db.q();
        self.db.update(DLInputs {
            e: !read_enabled,
            d: !self.int_ff.q(),
        });

        if old_db_q != self.db.q() {
            log::info!("DB NQ changed to {:?}", self.db.q());
        }

        if read_enabled.is_high() {
            // "DB7_Out" on the diagram. Set the top bit of the CPU data bus.
            let v = self.db.nq().is_high();
            log::info!("Updating top bit of data bus to {v}");
            cpu_pinout.data_bus = replacebits!(cpu_pinout.data_bus, "v... ....");
        }
    }
}

pub struct VbihInputs {
    pub pclk: SignalLevel,
    pub vset: SignalLevel,
    pub vbl: SignalLevel,
    pub vclr: SignalLevel,
    pub r2: SignalLevel,
    pub dbe: SignalLevel,
}

pub struct DLatch {
    q: SignalLevel,
}

impl DLatch {
    pub fn new(initial_q: SignalLevel) -> Self {
        Self { q: initial_q }
    }

    pub fn q(&self) -> SignalLevel {
        self.q
    }

    pub fn nq(&self) -> SignalLevel {
        !self.q
    }

    pub fn update(&mut self, inputs: DLInputs) {
        if inputs.e.is_high() {
            self.q = inputs.d;
        }
    }
}

#[derive(Clone, Copy)]
pub struct DLInputs {
    pub e: SignalLevel,
    pub d: SignalLevel,
}

// NOR-based
pub struct SRLatch {
    q: SignalLevel,
    nq: SignalLevel,
}

impl SRLatch {
    // Starting in the "invalid" state is currently unsupported.
    pub fn new(initial_q: SignalLevel) -> Self {
        Self {
            q: initial_q,
            nq: !initial_q,
        }
    }

    pub fn q(&self) -> SignalLevel {
        self.q
    }

    #[allow(dead_code)]
    pub fn nq(&self) -> SignalLevel {
        self.nq
    }

    pub fn update(&mut self, SRInputs { s, r }: SRInputs) {
        if s.is_high() {
            log::info!("Setting SR latch");
        }

        if r.is_high() {
            log::info!("Resetting SR latch");
        }

        if s.is_high() || r.is_high() {
            if s == r {
                log::info!("SR Invalid State!");
            }
            self.q = s & !r;
            self.nq = !s & r;
        }
    }
}

#[derive(Clone, Copy)]
pub struct SRInputs {
    pub s: SignalLevel,
    pub r: SignalLevel,
}
