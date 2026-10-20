use splitbits::replacebits;

use crate::bus::Bus;
use crate::circuit::d_latch::{DLInputs, DLatch};
use crate::circuit::sr_latch::{SRInputs, SRLatch};
use crate::memory::ppu::ppu_pinout::PpuPinout;
use crate::memory::signal_level::{ActiveLow, SignalLevel, nor, nor3};
use crate::ppu::ppu::Ppu;

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

    pub fn update(Bus { ppu_pinout, ppu_regs, ppu, cpu_pinout, cpu, ..}: &mut Bus) {
        let r2 = ppu_pinout.r2();
        let Ppu { vblank_interrupt_handler: vih, start_vblank: vset, end_vblank: vclr, .. } = ppu;
        let PpuPinout { pclk, data_bus_enabled: dbe, .. } = ppu_pinout;
        let vbl = if ppu_regs.nmi_enabled() { SignalLevel::High } else { SignalLevel::Low };
        let phi2 = *cpu.phi2;

        vih.vset_1.update(DLInputs {
            e: *pclk,
            d: *vset,
        });
        vih.vset_2.update(DLInputs {
            e: !*pclk,
            d: vih.vset_1.nq(),
        });
        let set = nor3(!*pclk, *vset, vih.vset_2.q());

        let read_enabled = nor(r2, *dbe);
        let reset = *vclr | read_enabled;

        vih.int_ff.update(SRInputs {
            s: set,
            r: reset,
        });

        let int = nor(!vbl, !vih.int_ff.q());
        cpu_pinout.nmi = !int;
        vih.nmip.update(DLInputs {
            e: phi2,
            d: cpu_pinout.nmi,
        });

        vih.db.update(DLInputs {
            e: !read_enabled,
            d: !vih.int_ff.q(),
        });

        if read_enabled.is_high() {
            // "DB7_Out" on the diagram. Set the top bit of the CPU data bus.
            let v = vih.db.nq().is_high();
            cpu_pinout.data_bus = replacebits!(cpu_pinout.data_bus, "v... ....");
        }
    }
}