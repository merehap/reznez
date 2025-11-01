use std::num::NonZeroI8;

use crate::mapper::*;

pub const IRQ_COUNTER: DirectlySetCounter = CounterBuilder::new()
    // The step is undefined at startup since the step is set at the same time as enabling the counter.
    .step(1)
    .wraps(false)
    .full_range(0, 0xFFFF)
    .initial_count(0)
    .auto_trigger_when(AutoTriggerWhen::StepSizedTransitionTo(0))
    .when_disabled_prevent(WhenDisabledPrevent::CountingAndTriggering)
    .build_directly_set_counter();

// Cony itself is not a mapper, but it forms the basis of the mapper 83 submappers.
pub struct Cony {
    irq_counter: DirectlySetCounter,
    next_irq_enabled_value: bool,
    scratch_ram: [u8; 4],
}

impl Cony {
    pub fn new() -> Self {
        Self {
            irq_counter: IRQ_COUNTER, 
            next_irq_enabled_value: false,
            scratch_ram: [0; 4],
        }
    }

    pub fn peek_cartridge_space(&self, mem: &Memory, addr: CpuAddress) -> ReadResult {
        if *addr & 0xDF00 == 0x5000 {
            ReadResult::partial_open_bus(mem.dip_switch, 0b0000_0011)
        } else if matches!(*addr & 0xDF03, 0x5100..=0x5FFF) {
            ReadResult::full(self.scratch_ram[usize::from(*addr & 0b11)])
        } else if *addr < 0x6000 {
            ReadResult::OPEN_BUS
        } else {
            mem.peek_prg(addr)
        }
    }

    // NOTE: CHR bank registers are handled differently by different submappers, so they are not handled here.
    pub fn write_register(&mut self, mem: &mut Memory, addr: CpuAddress, value: u8) {
        if matches!(*addr & 0xDF03, 0x5100..=0x5FFF) {
            self.scratch_ram[usize::from(*addr & 0b11)] = value;
        } else if *addr & 0x8300 == 0x8000 {
            // The w and o fields are shown, but not used, here. Only submapper 2 uses them.
            let fields = splitbits!(value, "wwoopppp");
            // The left shift here is not documented on the wiki, but it is necessary.
            // TODO: Probably need to store the same thing with the bottom bit dropped in P5 for 32KiB mode.
            mem.set_prg_register(P4, fields.p << 1);
        } else if *addr & 0x8300 == 0x8100 {
            // The "r" flag is shown, but not used, here. Submappers 0 and 2 use it.
            let fields = splitbits!(value, "esrll.mm");
            self.next_irq_enabled_value = fields.e;
            self.irq_counter.set_step(NonZeroI8::new(if fields.s { -1 } else { 1 }).unwrap());
            mem.set_prg_layout(fields.l);
            mem.set_name_table_mirroring(fields.m);
        } else if *addr & 0x8301 == 0x8200 {
            self.irq_counter.set_count_low_byte(value);
            mem.cpu_pinout.acknowledge_mapper_irq();
        } else if *addr & 0x8301 == 0x8201 {
            self.irq_counter.set_count_high_byte(value);
            self.irq_counter.set_enabled(self.next_irq_enabled_value);
        } else if matches!(*addr & 0x8313, 0x8300..=0x8302) {
            // P3 is not handled here since it is set differently in different submappers.
            let prg_id = [P0, P1, P2][usize::from(*addr & 0x8313) - 0x8300];
            mem.set_prg_register(prg_id, value);
        }
    }

    pub fn on_end_of_cpu_cycle(&mut self, mem: &mut Memory) {
        if self.irq_counter.tick().triggered {
            mem.cpu_pinout.assert_mapper_irq();
            self.irq_counter.disable();
        }
    }

    pub fn irq_counter_info(&self) -> Option<IrqCounterInfo> {
        Some(self.irq_counter.to_irq_counter_info())
    }
}