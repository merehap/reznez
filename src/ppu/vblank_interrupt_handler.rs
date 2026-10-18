use splitbits::replacebits;

use crate::circuit::d_latch::{DLInputs, DLatch};
use crate::circuit::sr_latch::{SRInputs, SRLatch};
use crate::memory::cpu::cpu_pinout::CpuPinout;
use crate::memory::signal_level::{ActiveHigh, ActiveLow, SignalLevel, nor, nor3};

// Reference: https://github.com/emu-russia/breaks/blob/master/BreakingNESWiki_DeepL/PPU/fsm.md#vblank-interrupt-handling
pub struct VBlankInterruptHandler {
    vset_1: DLatch,
    vset_2: DLatch,
    db: DLatch,
    int_ff: SRLatch,

    // Added here for convenience. Should maybe be split off.
    // TODO: No test ROMs exist for the nmip half-cycle delay.
    nmip: DLatch,
}

impl VBlankInterruptHandler {
    pub fn new() -> Self {
        Self {
            vset_1: DLatch::new(SignalLevel::High),
            vset_2: DLatch::new(SignalLevel::Low),
            db: DLatch::new(SignalLevel::High),
            int_ff: SRLatch::new(SignalLevel::Low),

            nmip: DLatch::new(SignalLevel::High),
        }
    }

    pub fn nmip(&self) -> ActiveLow {
        self.nmip.q().into()
    }

    pub fn update(&mut self, cpu_pinout: &mut CpuPinout, VbihInputs { pclk, vset, vbl, vclr, r2, dbe, phi2 }: VbihInputs) {
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
        self.nmip.update(DLInputs {
            e: *phi2,
            d: cpu_pinout.nmi,
        });

        self.db.update(DLInputs {
            e: !read_enabled,
            d: !self.int_ff.q(),
        });

        if read_enabled.is_high() {
            // "DB7_Out" on the diagram. Set the top bit of the CPU data bus.
            let v = self.db.nq().is_high();
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

    // Added here because it's being use for the separate circuit nmip circuit.
    pub phi2: ActiveHigh,
}