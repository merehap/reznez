use crate::bus::Bus;
use crate::cpu::cpu::Cpu;
use crate::memory::signal_level::{ActiveHigh, SignalLevel, nand, nor};

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

    pub fn update(bus: &mut Bus) {
        {
            let brk_ready = bus.cpu.step.brk5().and(!*bus.cpu.ready);
            let Cpu { interrupt_cycle_detector: icd, phi2, ready, ..} = &mut bus.cpu;
            if phi2.is_active() {
                icd.brk5 = brk_ready;
                icd.brk6_2 = !icd.brk6_1;
            } else {
                icd.brk6_1 = !*icd.brk5 & nand(**ready, !icd.brk6_1);
            }
        }

        bus.cpu.brk6e = InterruptCycleDetector::brk6e_signal(&mut bus.cpu);
        bus.cpu.brk7 = InterruptCycleDetector::brk7_signal(&bus.cpu)
    }

    fn brk6e_signal(Cpu { interrupt_cycle_detector: icd, ready, ..}: &Cpu) -> ActiveHigh {
        nor(!icd.brk6_2, **ready).into()
    }

    fn brk7_signal(Cpu { interrupt_cycle_detector: icd, ready, step, ..}: &Cpu) -> ActiveHigh {
        nor(!icd.brk6_1, *step.brk5().and(!*ready)).into()
    }
}