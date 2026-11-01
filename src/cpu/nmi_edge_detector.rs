use crate::bus::Bus;
use crate::cpu::b_flag::BFlag;
use crate::cpu::cpu::Cpu;
use crate::memory::signal_level::{ActiveLow, SignalLevel, nor3};

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
        self.ff1.nor(self.brk6e).nor(self.donmi).into()
    }

    pub fn update(bus: &mut Bus) {
        if bus.cpu.phi2.is_active() {
            let Cpu { nmi_edge_detector: ned, brk7, ..} = &mut bus.cpu;
            ned.brk7 = **brk7;

            let donmi = *ned.donmi_signal();
            ned.ff1 = donmi;
            ned.delay1 = donmi;

            ned.ff2 = ned.ff2.nor(ned.delay2).nor(ned.nmip);
        } else {
            let Cpu { nmi_edge_detector: ned, nmi_latch, brk6e, ..} = &mut bus.cpu;

            ned.nmip = *nmi_latch.nmip();
            ned.brk6e = **brk6e;
            ned.delay2 = !ned.delay1;

            let last_source = ned.ff2.nor(ned.delay2).nor(ned.nmip);
            ned.donmi = nor3(!ned.brk7, ned.nmip, last_source);
        }

        BFlag::update(bus); // Depends on BRK6E and DONMI. Also T0 and BR2 from the current CPU step.
    }
}