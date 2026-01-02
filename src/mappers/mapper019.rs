use crate::mapper::*;
use crate::memory::bank::bank::{ChrSourceRegisterId, WriteStatusRegisterId};
use crate::memory::memory::Memory;
use crate::memory::ppu::ciram::CiramSide;

const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(512 * KIBIBYTE)
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x67FF, 2 * KIBIBYTE, PrgBank::WORK_RAM.fixed_index(0).read_write_status(R12, W12)),
        PrgWindow::new(0x6800, 0x6FFF, 2 * KIBIBYTE, PrgBank::WORK_RAM.fixed_index(1).read_write_status(R13, W13)),
        PrgWindow::new(0x7000, 0x77FF, 2 * KIBIBYTE, PrgBank::WORK_RAM.fixed_index(2).read_write_status(R14, W14)),
        PrgWindow::new(0x7800, 0x7FFF, 2 * KIBIBYTE, PrgBank::WORK_RAM.fixed_index(3).read_write_status(R15, W15)),
        PrgWindow::new(0x8000, 0x9FFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P0)),
        PrgWindow::new(0xA000, 0xBFFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P1)),
        PrgWindow::new(0xC000, 0xDFFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P2)),
        PrgWindow::new(0xE000, 0xFFFF, 8 * KIBIBYTE, PrgBank::ROM.fixed_index(-1)),
    ])
    .chr_rom_max_size(256 * KIBIBYTE)
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x03FF, 1 * KIBIBYTE, ChrBank::with_switchable_source(CS0).switchable(C0).write_status(W0)),
        ChrWindow::new(0x0400, 0x07FF, 1 * KIBIBYTE, ChrBank::with_switchable_source(CS1).switchable(C1).write_status(W1)),
        ChrWindow::new(0x0800, 0x0BFF, 1 * KIBIBYTE, ChrBank::with_switchable_source(CS2).switchable(C2).write_status(W2)),
        ChrWindow::new(0x0C00, 0x0FFF, 1 * KIBIBYTE, ChrBank::with_switchable_source(CS3).switchable(C3).write_status(W3)),
        ChrWindow::new(0x1000, 0x13FF, 1 * KIBIBYTE, ChrBank::with_switchable_source(CS4).switchable(C4).write_status(W4)),
        ChrWindow::new(0x1400, 0x17FF, 1 * KIBIBYTE, ChrBank::with_switchable_source(CS5).switchable(C5).write_status(W5)),
        ChrWindow::new(0x1800, 0x1BFF, 1 * KIBIBYTE, ChrBank::with_switchable_source(CS6).switchable(C6).write_status(W6)),
        ChrWindow::new(0x1C00, 0x1FFF, 1 * KIBIBYTE, ChrBank::with_switchable_source(CS7).switchable(C7).write_status(W7)),
        ChrWindow::new(0x2000, 0x23FF, 1 * KIBIBYTE, ChrBank::with_switchable_source(NT0).switchable(N0).write_status(W8)),
        ChrWindow::new(0x2400, 0x27FF, 1 * KIBIBYTE, ChrBank::with_switchable_source(NT1).switchable(N1).write_status(W9)),
        ChrWindow::new(0x2800, 0x2BFF, 1 * KIBIBYTE, ChrBank::with_switchable_source(NT2).switchable(N2).write_status(W10)),
        ChrWindow::new(0x2C00, 0x2FFF, 1 * KIBIBYTE, ChrBank::with_switchable_source(NT3).switchable(N3).write_status(W11)),
    ])
    .fixed_name_table_mirroring()
    .build();

const IRQ_COUNTER: DirectlySetCounter = CounterBuilder::new()
    .step(1)
    .wraps(false)
    .full_range(0, 0x7FFF)
    .initial_count(0)
    // TODO: should this be only triggered by TransitionTo? Is an IRQ constantly being asserted until acknowledgement?
    .auto_trigger_when(AutoTriggerWhen::EndingOn(0x7FFF))
    // TODO: Test if this should be just Counting. Do counter reloads to 0x7FFF while disabled trigger IRQs?
    .when_disabled_prevent(WhenDisabledPrevent::CountingAndTriggering)
    .build_directly_set_counter();

// Namco 129 and Namco 163
// Needs testing, its IRQ was horribly broken when I found it, but might be fixed now.
pub struct Mapper019 {
    irq_counter: DirectlySetCounter,

    allow_ciram_in_low_chr: bool,
    allow_ciram_in_high_chr: bool,
}

impl Mapper for Mapper019 {
    fn peek_register(&self, _mem: &Memory, addr: CpuAddress) -> ReadResult {
        match *addr {
            0x0000..=0x401F | 0x6000..=0xFFFF => unreachable!(),
            0x4020..=0x47FF => ReadResult::OPEN_BUS,
            0x4800..=0x4FFF => /* TODO: Expansion Audio */ ReadResult::full(0),
            0x5000..=0x57FF => ReadResult::full(self.irq_counter.count_low_byte()),
            0x5800..=0x5FFF => ReadResult::full(self.irq_counter.count_high_byte()),
        }
    }

    fn write_register(&mut self, mem: &mut Memory, addr: CpuAddress, value: u8) {
        match *addr {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x47FF => { /* Do nothing. */ }
            0x4800..=0x4FFF => { /* TODO: Expansion Audio. */ }
            0x5000..=0x57FF => {
                mem.cpu_pinout.acknowledge_mapper_irq();
                self.irq_counter.set_count_low_byte(value);
            }
            0x5800..=0x5FFF => {
                mem.cpu_pinout.acknowledge_mapper_irq();

                let (irq_enable, irq_count_high_byte) = splitbits_named!(value, "eccccccc");
                self.irq_counter.set_count_high_byte(irq_count_high_byte);
                if irq_enable {
                    self.irq_counter.enable();
                } else {
                    self.irq_counter.disable();
                }
            }
            0x6000..=0x7FFF => { /* Do nothing. */ }
            0x8000..=0x87FF => set_chr_register(mem, self.allow_ciram_in_low_chr,  CS0, C0,   W0, value),
            0x8800..=0x8FFF => set_chr_register(mem, self.allow_ciram_in_low_chr,  CS1, C1,   W1, value),
            0x9000..=0x97FF => set_chr_register(mem, self.allow_ciram_in_low_chr,  CS2, C2,   W2, value),
            0x9800..=0x9FFF => set_chr_register(mem, self.allow_ciram_in_low_chr,  CS3, C3,   W3, value),
            0xA000..=0xA7FF => set_chr_register(mem, self.allow_ciram_in_high_chr, CS4, C4,   W4, value),
            0xA800..=0xAFFF => set_chr_register(mem, self.allow_ciram_in_high_chr, CS5, C5,   W5, value),
            0xB000..=0xB7FF => set_chr_register(mem, self.allow_ciram_in_high_chr, CS6, C6,   W6, value),
            0xB800..=0xBFFF => set_chr_register(mem, self.allow_ciram_in_high_chr, CS7, C7,   W7, value),
            0xC000..=0xC7FF => set_chr_register(mem, true,                         NT0, C8,   W8, value),
            0xC800..=0xCFFF => set_chr_register(mem, true,                         NT1, C9,   W9, value),
            0xD000..=0xD7FF => set_chr_register(mem, true,                         NT2, C10, W10, value),
            0xD800..=0xDFFF => set_chr_register(mem, true,                         NT3, C11, W11, value),
            0xE000..=0xE7FF => {
                // TODO: Pin 22 logic
                // TODO: Disable sound
                mem.set_prg_register(P0, value & 0b0011_1111);
            }
            0xE800..=0xEFFF => {
                let fields = splitbits!(value, "hlpp pppp");
                self.allow_ciram_in_high_chr = !fields.h;
                self.allow_ciram_in_low_chr = !fields.l;
                mem.set_prg_register(P1, fields.p);
            }
            0xF000..=0xF7FF => {
                // TODO: Pin 44 and PPU A12, A13
                mem.set_prg_register(P2, value & 0b0011_1111);
            }
            0xF800..=0xFFFF => {
                let fields = splitbits!(value, "ppppabcd");
                if fields.p == 0b0100 {
                    mem.set_writes_enabled(W0, fields.a);
                    mem.set_writes_enabled(W1, fields.b);
                    mem.set_writes_enabled(W2, fields.c);
                    mem.set_writes_enabled(W3, fields.d);
                } else {
                    // All read-only
                    mem.set_writes_enabled(W0, false);
                    mem.set_writes_enabled(W1, false);
                    mem.set_writes_enabled(W2, false);
                    mem.set_writes_enabled(W3, false);
                }
            }
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

fn set_chr_register(
    mem: &mut Memory,
    allow_ciram_in_chr: bool,
    source_reg_id: ChrSourceRegisterId,
    bank_reg_id: ChrBankRegisterId,
    status_reg_id: WriteStatusRegisterId,
    value: u8,
) {
    if allow_ciram_in_chr && value >= 0xE0 {
        let ciram_side = if value & 1 == 0 { CiramSide::Left } else { CiramSide::Right };
        // FIXME: Stop setting writes enabled/disabled? CIRAM should always have writes enabled (it should ignore status regs.)
        mem.set_chr_bank_register_to_ciram_side(source_reg_id, ciram_side);
        mem.set_writes_enabled(status_reg_id, true);
    } else {
        mem.set_chr_register(bank_reg_id, value);
        mem.set_writes_enabled(status_reg_id, false);
    }
}

impl Mapper019 {
    pub fn new() -> Self {
        Self {
            irq_counter: IRQ_COUNTER,
            allow_ciram_in_low_chr: true,
            allow_ciram_in_high_chr: true,
        }
    }
}
