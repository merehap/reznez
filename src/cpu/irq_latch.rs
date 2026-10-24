use crate::bus::Bus;
use crate::circuit::d_latch::{DLInputs, DLatch};
use crate::cpu::b_flag::BFlag;
use crate::memory::signal_level::{ActiveLow, SignalLevel};

pub struct IrqLatch {
    irq1: DLatch,
    irq2: DLatch,
}

impl IrqLatch {
    pub fn new() -> Self {
        Self {
            irq1: DLatch::new(SignalLevel::High),
            irq2: DLatch::new(SignalLevel::High),
        }
    }

    pub fn irqp(&self) -> ActiveLow {
        self.irq2.q().into()
    }

    pub fn update(bus: &mut Bus) {
        bus.cpu.irq_latch.irq1.update(DLInputs {
            e: *bus.cpu.phi2,
            d: *bus.cpu_pinout.irq(),
        });
        bus.cpu.irq_latch.irq2.update(DLInputs {
            e: !*bus.cpu.phi2,
            d: bus.cpu.irq_latch.irq1.q(),
        });
        BFlag::update(bus);
    }
}