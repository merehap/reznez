use crate::circuit::d_latch::{DLInputs, DLatch};
use crate::memory::signal_level::{ActiveHigh, ActiveLow, SignalLevel};

#[allow(dead_code)]
pub struct NmiLatch {
    nmi: DLatch,
}

#[allow(dead_code)]
impl NmiLatch {
    pub fn new() -> Self {
        Self {
            nmi: DLatch::new(SignalLevel::High),
        }
    }

    pub fn update(&mut self, inputs: NmiInputs) {
        self.nmi.update(DLInputs {
            e: *inputs.phi2,
            d: *inputs.nmi,
        });
    }
}

#[allow(dead_code)]
pub struct NmiInputs {
    pub nmi: ActiveLow,
    pub phi2: ActiveHigh,
}