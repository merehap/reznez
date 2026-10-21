use splitbits::replacebits;

use crate::bus::Bus;
use crate::circuit::d_latch::{DLInputs, DLatch};
use crate::circuit::sr_latch::{SRInputs, SRLatch};
use crate::memory::cpu::cpu_pinout::CpuPinout;
use crate::memory::signal_level::{SignalLevel, nor, nor3};
use crate::ppu::ppu::Ppu;

// Reference: https://github.com/emu-russia/breaks/blob/master/BreakingNESWiki_DeepL/PPU/fsm.md#vblank-interrupt-handling
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

    pub fn update(bus: &mut Bus) {
        Self::update_latches(bus);

        let vbl = if bus.ppu_regs.nmi_enabled() { SignalLevel::High } else { SignalLevel::Low };
        let int = nor(!vbl, !bus.ppu.vblank_interrupt_handler.int_ff.q());
        CpuPinout::set_nmi(bus, (!int).into());
    }

    fn update_latches(Bus { ppu_pinout, ppu, cpu_pinout, ..}: &mut Bus) {
        let r2 = ppu_pinout.r2();
        let Ppu { vblank_interrupt_handler: vih, start_vblank: vset, end_vblank: vclr, .. } = ppu;
        let pclk = ppu_pinout.pclk;
        let dbe = ppu_pinout.data_bus_enabled();

        vih.vset_1.update(DLInputs {
            e: pclk,
            d: *vset,
        });
        vih.vset_2.update(DLInputs {
            e: !pclk,
            d: vih.vset_1.nq(),
        });
        let set = nor3(!pclk, *vset, vih.vset_2.q());

        let read_enabled = nor(r2, *dbe);
        let reset = *vclr | read_enabled;

        vih.int_ff.update(SRInputs {
            s: set,
            r: reset,
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