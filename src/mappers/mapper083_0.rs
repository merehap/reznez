use std::num::NonZeroI8;

use crate::mapper::*;

const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(256 * KIBIBYTE)
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::ROM.switchable(P3).read_status(R0)),
        PrgWindow::new(0x8000, 0xBFFF, 16 * KIBIBYTE, PrgBank::ROM.switchable(P4)),
        PrgWindow::new(0xC000, 0xFFFF, 16 * KIBIBYTE, PrgBank::ROM.fixed_index(-1)),
    ])
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::ROM.switchable(P3).read_status(R0)),
        PrgWindow::new(0x8000, 0xFFFF, 32 * KIBIBYTE, PrgBank::ROM.switchable(P4)),
    ])
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::ROM.switchable(P3).read_status(R0)),
        PrgWindow::new(0x8000, 0x9FFF,  8 * KIBIBYTE, PrgBank::ROM.switchable(P0)),
        PrgWindow::new(0xA000, 0xBFFF,  8 * KIBIBYTE, PrgBank::ROM.switchable(P1)),
        PrgWindow::new(0xC000, 0xDFFF,  8 * KIBIBYTE, PrgBank::ROM.switchable(P2)),
        PrgWindow::new(0xE000, 0xFFFF,  8 * KIBIBYTE, PrgBank::ROM.fixed_index(-1)),
    ])
    // Same as above.
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::ROM.switchable(P3).read_status(R0)),
        PrgWindow::new(0x8000, 0x9FFF,  8 * KIBIBYTE, PrgBank::ROM.switchable(P0)),
        PrgWindow::new(0xA000, 0xBFFF,  8 * KIBIBYTE, PrgBank::ROM.switchable(P1)),
        PrgWindow::new(0xC000, 0xDFFF,  8 * KIBIBYTE, PrgBank::ROM.switchable(P2)),
        PrgWindow::new(0xE000, 0xFFFF,  8 * KIBIBYTE, PrgBank::ROM.fixed_index(-1)),
    ])
    .chr_rom_max_size(256 * KIBIBYTE)
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x03FF, 1 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C0)),
        ChrWindow::new(0x0400, 0x07FF, 1 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C1)),
        ChrWindow::new(0x0800, 0x0BFF, 1 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C2)),
        ChrWindow::new(0x0C00, 0x0FFF, 1 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C3)),
        ChrWindow::new(0x1000, 0x13FF, 1 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C4)),
        ChrWindow::new(0x1400, 0x17FF, 1 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C5)),
        ChrWindow::new(0x1800, 0x1BFF, 1 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C6)),
        ChrWindow::new(0x1C00, 0x1FFF, 1 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C7)),
    ])
    .name_table_mirrorings(&[
        NameTableMirroring::VERTICAL,
        NameTableMirroring::HORIZONTAL,
        NameTableMirroring::ONE_SCREEN_LEFT_BANK,
        NameTableMirroring::ONE_SCREEN_RIGHT_BANK,
    ])
    .build();

const IRQ_COUNTER: DirectlySetCounter = CounterBuilder::new()
    // The step is undefined at startup since the step is set at the same time as enabling the counter.
    .step(1)
    .full_range(0, 0xFFFF)
    .initial_count(0)
    .wraps(false)
    .auto_trigger_when(AutoTriggerWhen::StepSizedTransitionTo(0))
    .when_disabled_prevent(WhenDisabledPrevent::CountingAndTriggering)
    .build_directly_set_counter();

pub struct Mapper083_0 {
    irq_counter: DirectlySetCounter,
    next_irq_enabled_value: bool,
    scratch_ram: [u8; 4],
}

impl Mapper for Mapper083_0 {
    fn peek_cartridge_space(&self, mem: &Memory, addr: CpuAddress) -> ReadResult {
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

    fn write_register(&mut self, mem: &mut Memory, addr: CpuAddress, value: u8) {
        if matches!(*addr & 0xDF03, 0x5100..=0x5FFF) {
            self.scratch_ram[usize::from(*addr & 0b11)] = value;
        } else if *addr & 0x8300 == 0x8000 {
            mem.set_prg_register(P4, value & 0b1111);
        } else if *addr & 0x8300 == 0x8100 {
            let fields = splitbits!(value, "esrll.mm");
            self.next_irq_enabled_value = fields.e;
            self.irq_counter.set_step(NonZeroI8::new(if fields.s { -1 } else { 1 }).unwrap());
            mem.set_reads_enabled(R0, fields.r);
            mem.set_prg_layout(fields.l);
            mem.set_name_table_mirroring(fields.m);
        } else if *addr & 0x8301 == 0x8200 {
            self.irq_counter.set_count_low_byte(value);
            mem.cpu_pinout.acknowledge_mapper_irq();
        } else if *addr & 0x8301 == 0x8201 {
            self.irq_counter.set_count_high_byte(value);
            self.irq_counter.set_enabled(self.next_irq_enabled_value);
        } else if matches!(*addr & 0x8313, 0x8300..=0x8303) {
            let prg_id = [P0, P1, P2, P3][usize::from(*addr & 0x8313) - 0x8300];
            mem.set_prg_register(prg_id, value);
        } else if matches!(*addr & 0x831F, 0x8310..=0x8317) {
            let chr_id = [C0, C1, C2, C3, C4, C5, C6, C7][usize::from(*addr & 0x831F) - 0x8310];
            mem.set_chr_register(chr_id, value);
        }
    }

    fn on_end_of_cpu_cycle(&mut self, mem: &mut Memory) {
        if self.irq_counter.tick().triggered {
            mem.cpu_pinout.assert_mapper_irq();
        }
    }

    fn irq_counter_info(&self) -> Option<IrqCounterInfo> {
        Some(self.irq_counter.to_irq_counter_info())
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}

impl Mapper083_0 {
    pub fn new() -> Self {
        Self {
            irq_counter: IRQ_COUNTER, 
            next_irq_enabled_value: false,
            scratch_ram: [0; 4],
        }
    }
}