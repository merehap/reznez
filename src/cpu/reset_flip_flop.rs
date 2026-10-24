use crate::bus::Bus;
use crate::circuit::d_latch::{DLInputs, DLatch};
use crate::cpu::cpu::Cpu;
use crate::memory::signal_level::{ActiveHigh, SignalLevel};

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

    pub fn update(bus: &mut Bus) {
        let Cpu { reset_flip_flop: rff, rst_latch, phi2, brk6e, .. } = &mut bus.cpu;

        rff.res_1.update(DLInputs {
            e: **phi2,
            d: *rst_latch.resp(),
        });

        let not_dores = rff.res_1.q().nor(rff.res_2.q());
        rff.res_2.update(DLInputs {
            e: !**phi2,
            d: brk6e.nor(not_dores),
        });
    }
}