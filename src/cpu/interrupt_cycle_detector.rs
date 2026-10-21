use crate::bus::Bus;
use crate::memory::signal_level::{ActiveHigh, ActiveLow, SignalLevel, nand, nor};

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

    pub fn update(bus: &mut Bus) {
        let icd = &mut bus.cpu.interrupt_cycle_detector;
        if bus.cpu.phi2.is_active() {
            icd.brk5 = icd.brk5_rdy_signal(bus.cpu.step.brk5(), bus.cpu.ready);
            icd.brk6_2 = !icd.brk6_1;
        } else {
            icd.brk6_1 = !*icd.brk5 & nand(*bus.cpu.ready, !icd.brk6_1);
        }
    }
}