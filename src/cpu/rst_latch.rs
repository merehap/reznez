use crate::bus::Bus;
use crate::circuit::d_latch::{DLInputs, DLatch};
use crate::cpu::cpu::Cpu;
use crate::memory::signal_level::{ActiveHigh, SignalLevel};

pub struct RstLatch {
    rst_1: DLatch,
    rst_2: DLatch,
}

impl RstLatch {
    pub fn new() -> Self {
        Self {
            rst_1: DLatch::new(SignalLevel::Low),
            rst_2: DLatch::new(SignalLevel::Low),
        }
    }

    pub fn resp(&self) -> ActiveHigh {
        self.rst_2.q().into()
    }

    pub fn update(Bus { cpu, cpu_pinout, .. }: &mut Bus) {
        let Cpu { rst_latch, phi2, .. } = cpu;
        rst_latch.rst_1.update(DLInputs {
            e: **phi2,
            d: !*cpu_pinout.reset,
        });

        rst_latch.rst_2.update(DLInputs {
            e: **phi2,
            d: rst_latch.rst_1.q(),
        });
    }
}