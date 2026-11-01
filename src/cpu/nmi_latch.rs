use crate::bus::Bus;
use crate::circuit::d_latch::{DLInputs, DLatch};
use crate::cpu::nmi_edge_detector::NmiEdgeDetector;
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

    pub fn update(bus: &mut Bus) {
        bus.cpu.nmi_latch.nmi.update(DLInputs {
            e: *bus.cpu.phi2,
            d: *bus.cpu_pinout.nmi(),
        });
        NmiEdgeDetector::update(bus);
    }
}