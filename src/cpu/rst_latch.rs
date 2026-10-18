use crate::bus::Bus;
use crate::circuit::d_latch::{DLInputs, DLatch};
use crate::memory::signal_level::{ActiveHigh, ActiveLow, SignalLevel};

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

    pub fn update(&mut self, RstInputs { rst: res, phi2 }: RstInputs) {
        self.rst_1.update(DLInputs {
            e: *phi2,
            d: !*res,
        });

        self.rst_2.update(DLInputs {
            e: *phi2,
            d: self.rst_1.q(),
        });
    }
}

pub struct RstInputs {
    rst: ActiveLow,
    phi2: ActiveHigh,
}

impl RstInputs {
    pub fn new(Bus { cpu, cpu_pinout, ..}: &Bus) -> Self {
        Self {
            rst: cpu_pinout.reset,
            phi2: cpu.phi2,
        }
    }
}