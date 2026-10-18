use crate::bus::Bus;
use crate::circuit::d_latch::{DLInputs, DLatch};
use crate::cpu::cpu::Cpu;
use crate::memory::signal_level::{ActiveHigh, SignalLevel, nor};

pub struct ResetFlipFlop {
    res_1: DLatch,
    res_2: DLatch,
}

impl ResetFlipFlop {
    pub fn new() -> Self {
        Self {
            res_1: DLatch::new(SignalLevel::Low),
            res_2: DLatch::new(SignalLevel::Low),
        }
    }

    pub fn dores(&self) -> ActiveHigh {
        (self.res_1.q() | self.res_2.q()).into()
    }

    pub fn update(&mut self, ResetInputs { resp, phi2, brk6e }: ResetInputs) {
        self.res_1.update(DLInputs {
            e: *phi2,
            d: *resp,
        });

        let not_dores = nor(self.res_1.q(), self.res_2.q());
        self.res_2.update(DLInputs {
            e: !*phi2,
            d: nor(not_dores, *brk6e),
        });
    }
}

pub struct ResetInputs {
    resp: ActiveHigh,
    phi2: ActiveHigh,
    brk6e: ActiveHigh,
}

impl ResetInputs {
    pub fn new(Bus { cpu, ..}: &Bus) -> Self {
        Self::from_components(cpu)
    }

    pub fn from_components(cpu: &Cpu) -> Self {
        Self {
            resp: cpu.rst_latch.resp(),
            phi2: cpu.phi2,
            brk6e: cpu.interrupt_cycle_detector.brk6e_signal(cpu.ready),
        }
    }
}