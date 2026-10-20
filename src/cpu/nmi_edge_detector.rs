use crate::bus::Bus;
use crate::cpu::cpu::Cpu;
use crate::memory::signal_level::{ActiveLow, SignalLevel, nor, nor3};
use crate::ppu::ppu::Ppu;

// Questions:
// * Should this all be processed right on the clock transition?
// https://github.com/emu-russia/breaks/blob/master/BreakingNESWiki_DeepL/6502/interrupts.md
#[derive(Clone, Copy)]
pub struct NmiEdgeDetector {
    // D latches
    nmip: SignalLevel,
    brk6e: SignalLevel,
    brk7: SignalLevel,
    ff1: SignalLevel,
    ff2: SignalLevel,
    delay1: SignalLevel,
    delay2: SignalLevel,
    donmi: SignalLevel,
}

impl NmiEdgeDetector {
    pub fn new() -> Self {
        Self {
            nmip: SignalLevel::High,
            brk6e: SignalLevel::Low,
            brk7: SignalLevel::High, // FIXME: This value is probably wrong.
            ff1: SignalLevel::High, // NMI would trigger immediately after RESET if this started Low.
            ff2: SignalLevel::Low,
            delay1: SignalLevel::High,
            delay2: SignalLevel::Low,
            donmi: SignalLevel::Low,
        }
    }

    pub fn donmi_signal(self) -> ActiveLow {
        nor(self.donmi, nor(self.ff1, self.brk6e)).into()
    }

    pub fn step_phi1(bus: &mut Bus) {
        let Cpu { nmi_edge_detector: ned, interrupt_cycle_detector, ready, ..} = &mut bus.cpu;
        let Ppu { vblank_interrupt_handler, ..} = &mut bus.ppu;

        ned.nmip = *vblank_interrupt_handler.nmip();
        ned.brk6e = *interrupt_cycle_detector.brk6e_signal(*ready);
        ned.delay2 = !ned.delay1;

        let last_source = nor(ned.nmip, nor(ned.ff2, ned.delay2));
        ned.donmi = nor3(!ned.brk7, ned.nmip, last_source);
    }

    pub fn step_phi2(bus: &mut Bus) {
        let Cpu { nmi_edge_detector: ned, interrupt_cycle_detector, ready, step, ..} = &mut bus.cpu;

        ned.brk7 = interrupt_cycle_detector.brk7_signal(step.brk5(), *ready);

        let donmi = *ned.donmi_signal();
        ned.ff1 = donmi;
        ned.delay1 = donmi;

        ned.ff2 = nor(ned.nmip, nor(ned.ff2, ned.delay2));
    }
}