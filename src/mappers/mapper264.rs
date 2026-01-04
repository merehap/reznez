use std::num::NonZeroI8;

use crate::mapper::*;
use crate::mappers::common::cony;

// Identical to submapper 0 layout, except with PRG work ram and PRG and CHR outer banks.
const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(256 * KIBIBYTE)
    .prg_rom_outer_bank_size(128 * KIBIBYTE)
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::ABSENT),
        PrgWindow::new(0x8000, 0xBFFF, 16 * KIBIBYTE, PrgBank::ROM.switchable(P4)),
        PrgWindow::new(0xC000, 0xFFFF, 16 * KIBIBYTE, PrgBank::ROM.fixed_index(-1)),
    ])
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::ABSENT),
        PrgWindow::new(0x8000, 0xFFFF, 32 * KIBIBYTE, PrgBank::ROM.switchable(P4)),
    ])
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::ABSENT),
        PrgWindow::new(0x8000, 0x9FFF,  8 * KIBIBYTE, PrgBank::ROM.switchable(P0)),
        PrgWindow::new(0xA000, 0xBFFF,  8 * KIBIBYTE, PrgBank::ROM.switchable(P1)),
        PrgWindow::new(0xC000, 0xDFFF,  8 * KIBIBYTE, PrgBank::ROM.switchable(P2)),
        PrgWindow::new(0xE000, 0xFFFF,  8 * KIBIBYTE, PrgBank::ROM.switchable(P3)),
    ])
    // Same as above.
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::ABSENT),
        PrgWindow::new(0x8000, 0x9FFF,  8 * KIBIBYTE, PrgBank::ROM.switchable(P0)),
        PrgWindow::new(0xA000, 0xBFFF,  8 * KIBIBYTE, PrgBank::ROM.switchable(P1)),
        PrgWindow::new(0xC000, 0xDFFF,  8 * KIBIBYTE, PrgBank::ROM.switchable(P2)),
        PrgWindow::new(0xE000, 0xFFFF,  8 * KIBIBYTE, PrgBank::ROM.switchable(P3)),
    ])
    .override_prg_bank_register(P3, -1)
    .chr_rom_max_size(512 * KIBIBYTE)
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x07FF, 2 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C0)),
        ChrWindow::new(0x0800, 0x0FFF, 2 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C1)),
        ChrWindow::new(0x1000, 0x17FF, 2 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C2)),
        ChrWindow::new(0x1800, 0x1FFF, 2 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C3)),
    ])
    .name_table_mirrorings(&[
        NameTableMirroring::VERTICAL,
        NameTableMirroring::HORIZONTAL,
        NameTableMirroring::ONE_SCREEN_LEFT_BANK,
        NameTableMirroring::ONE_SCREEN_RIGHT_BANK,
    ])
    .build();

// Very similar to other Cony mappers, but different enough that sharing most code isn't worth it.
pub struct Mapper264 {
    irq_counter: DirectlySetCounter,
    next_irq_enabled_value: bool,
    scratch_ram: [u8; 4],
}

impl Mapper for Mapper264 {
    fn peek_register(&self, bus: &Bus, addr: CpuAddress) -> ReadResult {
        if *addr & 0xD400 == 0x5000 {
            ReadResult::partial(bus.dip_switch, 0b0000_0011)
        } else if matches!(*addr & 0xD403, 0x5400..=0x5403) {
            ReadResult::full(self.scratch_ram[usize::from((*addr & 0xD403) - 0x5400)])
        } else {
            ReadResult::OPEN_BUS
        }
    }

    fn write_register(&mut self, bus: &mut Bus, addr: CpuAddress, value: u8) {
        if matches!(*addr & 0xD403, 0x5400..=0x5403) {
            self.scratch_ram[usize::from((*addr & 0xD403) - 0x5400)] = value;
            return;
        }

        match *addr & 0x8C17 {
            0x8000 => {
                let fields = splitbits!(min=u8, value, "....oppp");
                bus.set_prg_rom_outer_bank_number(fields.o);
                // TODO: Verify correct behavior for both 16KiB and 32KiB layouts.
                bus.set_prg_register(P4, fields.p);
            }
            0x8400 => {
                let fields = splitbits!(value, "es.ll.mm");
                self.next_irq_enabled_value = fields.e;
                self.irq_counter.set_step(NonZeroI8::new(if fields.s { -1 } else { 1 }).unwrap());
                bus.set_prg_layout(fields.l);
                bus.set_name_table_mirroring(fields.m);
            }
            0x8800 => {
                self.irq_counter.set_count_low_byte(value);
                bus.cpu_pinout.acknowledge_mapper_irq();
            }
            0x8801 => {
                self.irq_counter.set_count_high_byte(value);
                self.irq_counter.set_enabled(self.next_irq_enabled_value);
            }
            0x8C00 => bus.set_prg_register(P0, value & 0b1111),
            0x8C01 => bus.set_prg_register(P1, value & 0b1111),
            0x8C02 => bus.set_prg_register(P2, value & 0b1111),
            0x8C03 => bus.set_prg_register(P3, value & 0b1111),
            0x8C10 => bus.set_chr_register(C0, value),
            0x8C11 => bus.set_chr_register(C1, value),
            0x8C16 => bus.set_chr_register(C2, value),
            0x8C17 => bus.set_chr_register(C3, value),
            _ => { /* Do nothing. */ }
        }
    }

    fn on_end_of_cpu_cycle(&mut self, bus: &mut Bus) {
        if self.irq_counter.tick().triggered {
            bus.cpu_pinout.assert_mapper_irq();
            self.irq_counter.disable();
        }
    }

    fn irq_counter_info(&self) -> Option<IrqCounterInfo> {
        Some(self.irq_counter.to_irq_counter_info())
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}

impl Mapper264 {
    pub fn new() -> Self {
        Self {
            irq_counter: cony::IRQ_COUNTER,
            next_irq_enabled_value: false,
            scratch_ram: [0; 4],
        }
    }
}