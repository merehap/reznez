use crate::bus::Bus;
use crate::circuit::d_latch::DLInputs;
use crate::circuit::d_latch::DLatch;
use crate::cpu::cpu::Cpu;
use crate::cpu::reset_flip_flop::ResetFlipFlop;
use crate::memory::signal_level::ActiveHigh;
use crate::memory::signal_level::SignalLevel;

pub struct BFlag {
    b1: DLatch,
    b2: DLatch,
}

impl BFlag {
    pub fn new() -> Self {
        Self {
            // TODO: Verify these initial values, not that they matter.
            b1: DLatch::new(SignalLevel::Low),
            b2: DLatch::new(SignalLevel::Low),
        }
    }

    pub fn b_out(&self, reset_flip_flop: &ResetFlipFlop, brk6e: ActiveHigh) -> ActiveHigh {
        //log::info!("b1: {:?}, brk6e: {:?}, dores: {:?}, b2: {:?}", self.b1.q(), *brk6e, *(reset_flip_flop.dores()), self.b2.q());
        self.b1.q()
            .nor(brk6e)
            .nor(reset_flip_flop.dores()).into()
    }

    pub fn update(bus: &mut Bus) {
        Self::update_latches(&mut bus.cpu);
    }

    // TODO: Optimize
    fn update_latches(cpu: &mut Cpu) {
        let br2 = cpu.br2();

        let i_out = if cpu.status().interrupts_disabled() { SignalLevel::High } else { SignalLevel::Low };
        let Cpu { b_flag, irq_latch, nmi_edge_detector, phi2, brk6e, step, ..} = cpu;
        b_flag.b2.update(DLInputs {
            e: !**phi2,
            d: b_flag.b1.q().nor(*brk6e),
        });

        let br2_t0 = br2.or(step.t0());
        let irqp_i_out = (*irq_latch.irqp()) | i_out;
        let b_flag_input = br2_t0.nand(nmi_edge_detector.donmi_signal()).nand(irqp_i_out);

        b_flag.b1.update(DLInputs {
            e: **phi2,
            d: b_flag.b2.nq() & b_flag_input,
        });
    }
}