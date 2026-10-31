use crate::bus::Bus;
use crate::circuit::d_latch::{DLInputs, DLatch};
use crate::cpu::reset_flip_flop::ResetFlipFlop;
use crate::memory::cpu::cpu_pinout::CpuPinout;
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

    pub fn update(bus: &mut Bus) {
        bus.cpu.rst_latch.update_latches(&mut bus.cpu_pinout, bus.cpu.phi2);
        ResetFlipFlop::update(bus); // RESP changed
    }

    fn update_latches(&mut self, cpu_pinout: &mut CpuPinout, phi2: ActiveHigh) {
        self.rst_1.update(DLInputs {
            e: *phi2,
            d: !*cpu_pinout.reset(),
        });

        self.rst_2.update(DLInputs {
            e: !*phi2,
            d: self.rst_1.q(),
        });
    }
}