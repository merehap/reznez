use crate::{bus::Bus, cpu::cpu::Cpu, memory::signal_level::{ActiveHigh, ActiveLow, SignalLevel, nand, nor}};

#[derive(Clone, Copy)]
pub struct InterruptCycleDetector {
    // D latches
    brk5: ActiveHigh,
    brk6_1: SignalLevel,
    brk6_2: SignalLevel,
}

impl InterruptCycleDetector {
    pub fn new() -> Self {
        Self {
            brk5: ActiveHigh::Inactive,
            brk6_1: SignalLevel::High,
            brk6_2: SignalLevel::Low,
        }
    }

    pub fn brk5_rdy_signal(self, brk5_signal: ActiveHigh, ready_signal: ActiveLow) -> ActiveHigh {
        brk5_signal.and(!ready_signal)
    }

    pub fn brk6e_signal(self, ready_signal: ActiveLow) -> ActiveHigh {
        nor(!self.brk6_2, *ready_signal).into()
    }

    pub fn brk7_signal(self, brk5_signal: ActiveHigh, ready_signal: ActiveLow) -> SignalLevel {
        nor(!self.brk6_1, *self.brk5_rdy_signal(brk5_signal, ready_signal))
    }

    pub fn step_phi1(bus: &mut Bus) {
        let Cpu { interrupt_cycle_detector: icd, ready, .. } = &mut bus.cpu;
        icd.brk6_1 = !*icd.brk5 & nand(**ready, !icd.brk6_1);
    }

    pub fn step_phi2(&mut self, brk5_signal: ActiveHigh, ready_signal: ActiveLow) {
        self.brk5 = self.brk5_rdy_signal(brk5_signal, ready_signal);
        self.brk6_2 = !self.brk6_1;
    }
}