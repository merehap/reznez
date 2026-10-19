use crate::{bus::Bus, circuit::d_latch::DLatch, memory::signal_level::{ActiveHigh, ActiveLow, SignalLevel}};

#[allow(dead_code)]
pub struct IrqLatch {
    irq1: DLatch,
    irq2: DLatch,
}

#[allow(dead_code)]
impl IrqLatch {
    pub fn new() -> Self {
        Self {
            irq1: DLatch::new(SignalLevel::High),
            irq2: DLatch::new(SignalLevel::High),
        }
    }


}

#[allow(dead_code)]
pub struct IrqInputs {
    irq: ActiveLow,
    phi2: ActiveHigh,
}

#[allow(dead_code)]
impl IrqInputs {
    pub fn new(Bus { cpu, cpu_pinout, .. }: &Bus) -> Self {
        Self {
            irq: cpu_pinout.irq(),
            phi2: cpu.phi2,
        }
    }
}