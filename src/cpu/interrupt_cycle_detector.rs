use crate::memory::signal_level::{SignalLevel, nand, nor};

#[derive(Clone, Copy)]
pub struct InterruptCycleDetector {
    // D latches
    brk5: SignalLevel,
    brk6_1: SignalLevel,
    brk6_2: SignalLevel,
}

impl InterruptCycleDetector {
    pub fn new() -> Self {
        Self {
            brk5: SignalLevel::Low,
            brk6_1: SignalLevel::Low,
            brk6_2: SignalLevel::High,
        }
    }

    pub fn brk5_rdy_signal(self, brk5_signal: SignalLevel, ready_signal: SignalLevel) -> SignalLevel {
        brk5_signal & !ready_signal
    }

    pub fn brk6e_signal(self, ready_signal: SignalLevel) -> SignalLevel {
        nor(!self.brk6_2, ready_signal)
    }

    pub fn brk7_signal(self, brk5_signal: SignalLevel, ready_signal: SignalLevel) ->  SignalLevel {
        nor(!self.brk6_1, self.brk5_rdy_signal(brk5_signal, ready_signal))
    }

    pub fn step_phi1(&mut self, ready_signal: SignalLevel) {
        self.brk6_1 = !self.brk5 & nand(ready_signal, !self.brk6_1);
    }

    pub fn step_phi2(&mut self, brk5_signal: SignalLevel, ready_signal: SignalLevel) {
        self.brk5 = self.brk5_rdy_signal(brk5_signal, ready_signal);
        self.brk6_2 = !self.brk6_1;
    }
}