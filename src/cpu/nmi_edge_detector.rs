use crate::{cpu::interrupt_cycle_detector::InterruptCycleDetector, memory::signal_level::{SignalLevel, nor, nor3}};

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
            brk7: SignalLevel::High,
            ff1: SignalLevel::High, // NMI would trigger immediately after RESET if this started Low.
            ff2: SignalLevel::Low,
            delay1: SignalLevel::High,
            delay2: SignalLevel::Low,
            donmi: SignalLevel::Low,
        }
    }

    pub fn donmi_signal(self) -> SignalLevel {
        nor(self.donmi, nor(self.ff1, self.brk6e))
    }

    pub fn step_phi1(&mut self, cycle_detector: &InterruptCycleDetector, nmip_signal: SignalLevel, ready_signal: SignalLevel) {
        self.nmip = nmip_signal;
        self.brk6e = cycle_detector.brk6e_signal(ready_signal);
        self.delay2 = !self.delay1;

        let last_source = nor(self.nmip, nor(self.ff2, self.delay2));
        self.donmi = nor3(!self.brk7, nmip_signal, last_source);
        log::info!("NMIP {:?}, BRK6E {:?}, BRK7 {:?}, DONMI {:?}", self.nmip, self.brk6e, self.brk7, self.donmi);
    }

    pub fn step_phi2(&mut self, cycle_detector: &InterruptCycleDetector, brk5_signal: SignalLevel, ready_signal: SignalLevel) {
        self.brk7 = cycle_detector.brk7_signal(brk5_signal, ready_signal);

        let donmi_low = self.donmi_signal();
        self.ff1 = donmi_low;
        self.delay1 = donmi_low;

        self.ff2 = nor(self.nmip, nor(self.ff2, self.delay2));

        log::info!("NMIP {:?}, BRK6E {:?}, BRK7 {:?}, DONMI {:?}", self.nmip, self.brk6e, self.brk7, self.donmi);
    }
}