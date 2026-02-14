use crate::mapper::*;
use crate::memory::bank::bank::{ChrSourceRegisterId, WriteStatusRegisterId};
use crate::memory::ppu::ciram::CiramSide;

const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(512 * KIBIBYTE)
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x67FF, 2 * KIBIBYTE, PrgBank::RAM_OR_ABSENT.fixed_number(0).read_write_status(RS12, WS12)),
        PrgWindow::new(0x6800, 0x6FFF, 2 * KIBIBYTE, PrgBank::RAM_OR_ABSENT.fixed_number(1).read_write_status(RS13, WS13)),
        PrgWindow::new(0x7000, 0x77FF, 2 * KIBIBYTE, PrgBank::RAM_OR_ABSENT.fixed_number(2).read_write_status(RS14, WS14)),
        PrgWindow::new(0x7800, 0x7FFF, 2 * KIBIBYTE, PrgBank::RAM_OR_ABSENT.fixed_number(3).read_write_status(RS15, WS15)),
        PrgWindow::new(0x8000, 0x9FFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P)),
        PrgWindow::new(0xA000, 0xBFFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(Q)),
        PrgWindow::new(0xC000, 0xDFFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(R)),
        PrgWindow::new(0xE000, 0xFFFF, 8 * KIBIBYTE, PrgBank::ROM.fixed_number(-1)),
    ])
    .chr_rom_max_size(256 * KIBIBYTE)
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x03FF, 1 * KIBIBYTE, ChrBank::with_switchable_source(CS0).switchable(C).write_status(WS0)),
        ChrWindow::new(0x0400, 0x07FF, 1 * KIBIBYTE, ChrBank::with_switchable_source(CS1).switchable(D).write_status(WS1)),
        ChrWindow::new(0x0800, 0x0BFF, 1 * KIBIBYTE, ChrBank::with_switchable_source(CS2).switchable(E).write_status(WS2)),
        ChrWindow::new(0x0C00, 0x0FFF, 1 * KIBIBYTE, ChrBank::with_switchable_source(CS3).switchable(F).write_status(WS3)),
        ChrWindow::new(0x1000, 0x13FF, 1 * KIBIBYTE, ChrBank::with_switchable_source(CS4).switchable(G).write_status(WS4)),
        ChrWindow::new(0x1400, 0x17FF, 1 * KIBIBYTE, ChrBank::with_switchable_source(CS5).switchable(H).write_status(WS5)),
        ChrWindow::new(0x1800, 0x1BFF, 1 * KIBIBYTE, ChrBank::with_switchable_source(CS6).switchable(I).write_status(WS6)),
        ChrWindow::new(0x1C00, 0x1FFF, 1 * KIBIBYTE, ChrBank::with_switchable_source(CS7).switchable(J).write_status(WS7)),
        ChrWindow::new(0x2000, 0x23FF, 1 * KIBIBYTE, ChrBank::with_switchable_source(NTS0).switchable(NT0).write_status(WS8)),
        ChrWindow::new(0x2400, 0x27FF, 1 * KIBIBYTE, ChrBank::with_switchable_source(NTS1).switchable(NT1).write_status(WS9)),
        ChrWindow::new(0x2800, 0x2BFF, 1 * KIBIBYTE, ChrBank::with_switchable_source(NTS2).switchable(NT2).write_status(WS10)),
        ChrWindow::new(0x2C00, 0x2FFF, 1 * KIBIBYTE, ChrBank::with_switchable_source(NTS3).switchable(NT3).write_status(WS11)),
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
    fn peek_register(&self, _bus: &Bus, addr: CpuAddress) -> ReadResult {
        match *addr {
            0x0000..=0x401F | 0x6000..=0xFFFF => unreachable!(),
            0x4020..=0x47FF => ReadResult::OPEN_BUS,
            0x4800..=0x4FFF => /* TODO: Expansion Audio */ ReadResult::full(0),
            0x5000..=0x57FF => ReadResult::full(self.irq_counter.count_low_byte()),
            0x5800..=0x5FFF => ReadResult::full(self.irq_counter.count_high_byte()),
        }
    }

    fn write_register(&mut self, bus: &mut Bus, addr: CpuAddress, value: u8) {
        match *addr {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x47FF => { /* Do nothing. */ }
            0x4800..=0x4FFF => { /* TODO: Expansion Audio. */ }
            0x5000..=0x57FF => {
                bus.cpu_pinout.acknowledge_mapper_irq();
                self.irq_counter.set_count_low_byte(value);
            }
            0x5800..=0x5FFF => {
                bus.cpu_pinout.acknowledge_mapper_irq();

                let (irq_enable, irq_count_high_byte) = splitbits_named!(value, "eccccccc");
                self.irq_counter.set_count_high_byte(irq_count_high_byte);
                if irq_enable {
                    self.irq_counter.enable();
                } else {
                    self.irq_counter.disable();
                }
            }
            0x6000..=0x7FFF => { /* Do nothing. */ }
            0x8000..=0x87FF => set_chr_register(bus, self.allow_ciram_in_low_chr,  CS0, C,   WS0, value),
            0x8800..=0x8FFF => set_chr_register(bus, self.allow_ciram_in_low_chr,  CS1, D,   WS1, value),
            0x9000..=0x97FF => set_chr_register(bus, self.allow_ciram_in_low_chr,  CS2, E,   WS2, value),
            0x9800..=0x9FFF => set_chr_register(bus, self.allow_ciram_in_low_chr,  CS3, F,   WS3, value),
            0xA000..=0xA7FF => set_chr_register(bus, self.allow_ciram_in_high_chr, CS4, G,   WS4, value),
            0xA800..=0xAFFF => set_chr_register(bus, self.allow_ciram_in_high_chr, CS5, H,   WS5, value),
            0xB000..=0xB7FF => set_chr_register(bus, self.allow_ciram_in_high_chr, CS6, I,   WS6, value),
            0xB800..=0xBFFF => set_chr_register(bus, self.allow_ciram_in_high_chr, CS7, J,   WS7, value),
            // FIXME: C8 through C11 aren't used here anymore. Switch to N0 through N3 instead, then test.
            0xC000..=0xC7FF => set_chr_register(bus, true,                         NTS0, K,   WS8, value),
            0xC800..=0xCFFF => set_chr_register(bus, true,                         NTS1, L,   WS9, value),
            0xD000..=0xD7FF => set_chr_register(bus, true,                         NTS2, M, WS10, value),
            0xD800..=0xDFFF => set_chr_register(bus, true,                         NTS3, N, WS11, value),
            0xE000..=0xE7FF => {
                // TODO: Pin 22 logic
                // TODO: Disable sound
                bus.set_prg_register(P, value & 0b0011_1111);
            }
            0xE800..=0xEFFF => {
                let fields = splitbits!(value, "hlpp pppp");
                self.allow_ciram_in_high_chr = !fields.h;
                self.allow_ciram_in_low_chr = !fields.l;
                bus.set_prg_register(Q, fields.p);
            }
            0xF000..=0xF7FF => {
                // TODO: Pin 44 and PPU A12, A13
                bus.set_prg_register(R, value & 0b0011_1111);
            }
            0xF800..=0xFFFF => {
                let fields = splitbits!(value, "ppppabcd");
                if fields.p == 0b0100 {
                    bus.set_writes_enabled(WS0, fields.a);
                    bus.set_writes_enabled(WS1, fields.b);
                    bus.set_writes_enabled(WS2, fields.c);
                    bus.set_writes_enabled(WS3, fields.d);
                } else {
                    // All read-only
                    bus.set_writes_enabled(WS0, false);
                    bus.set_writes_enabled(WS1, false);
                    bus.set_writes_enabled(WS2, false);
                    bus.set_writes_enabled(WS3, false);
                }
            }
        }
    }

    fn on_end_of_cpu_cycle(&mut self, bus: &mut Bus) {
        if self.irq_counter.tick().triggered {
            bus.cpu_pinout.assert_mapper_irq();
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
    bus: &mut Bus,
    allow_ciram_in_chr: bool,
    source_reg_id: ChrSourceRegisterId,
    bank_reg_id: ChrBankRegisterId,
    status_reg_id: WriteStatusRegisterId,
    value: u8,
) {
    if allow_ciram_in_chr && value >= 0xE0 {
        let ciram_side = if value & 1 == 0 { CiramSide::Left } else { CiramSide::Right };
        // FIXME: Stop setting writes enabled/disabled? CIRAM should always have writes enabled (it should ignore status regs.)
        bus.set_chr_bank_register_to_ciram_side(source_reg_id, ciram_side);
        bus.set_writes_enabled(status_reg_id, true);
    } else {
        bus.set_chr_register(bank_reg_id, value);
        bus.set_writes_enabled(status_reg_id, false);
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
