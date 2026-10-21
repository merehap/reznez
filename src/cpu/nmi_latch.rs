use crate::bus::Bus;
use crate::circuit::d_latch::{DLInputs, DLatch};
use crate::memory::signal_level::{ActiveLow, SignalLevel};

pub struct NmiLatch {
    nmi: DLatch,
}

impl NmiLatch {
    pub fn new() -> Self {
        Self {
            nmi: DLatch::new(SignalLevel::High),
        }
    }

    pub fn nmip(&self) -> ActiveLow {
        self.nmi.q().into()
    }

    pub fn update(Bus { cpu, cpu_pinout, ..}: &mut Bus) {
        cpu.nmi_latch.nmi.update(DLInputs {
            e: *cpu.phi2,
            d: *cpu_pinout.nmi(),
        });
    }
}