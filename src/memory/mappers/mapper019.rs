use crate::memory::mapper::*;
use crate::memory::ppu::vram::VramSide;

const LAYOUT: Layout = Layout::builder()
    .prg_max_size(128 * KIBIBYTE)
    .prg_layout(&[
        Window::new(0x6000, 0x67FF, 2 * KIBIBYTE, Bank::WORK_RAM.status_register(S12)),
        Window::new(0x6800, 0x6FFF, 2 * KIBIBYTE, Bank::WORK_RAM.status_register(S13)),
        Window::new(0x7000, 0x77FF, 2 * KIBIBYTE, Bank::WORK_RAM.status_register(S14)),
        Window::new(0x7800, 0x7FFF, 2 * KIBIBYTE, Bank::WORK_RAM.status_register(S15)),
        Window::new(0x8000, 0x9FFF, 8 * KIBIBYTE, Bank::ROM.switchable(P0)),
        Window::new(0xA000, 0xBFFF, 8 * KIBIBYTE, Bank::ROM.switchable(P1)),
        Window::new(0xC000, 0xDFFF, 8 * KIBIBYTE, Bank::ROM.switchable(P2)),
        Window::new(0xE000, 0xFFFF, 8 * KIBIBYTE, Bank::ROM.fixed_index(-1)),
    ])
    .chr_max_size(32 * KIBIBYTE)
    .chr_layout(&[
        Window::new(0x0000, 0x03FF, 1 * KIBIBYTE, Bank::RAM.switchable(C0).status_register(S0)),
        Window::new(0x0400, 0x07FF, 1 * KIBIBYTE, Bank::RAM.switchable(C1).status_register(S1)),
        Window::new(0x0800, 0x0BFF, 1 * KIBIBYTE, Bank::RAM.switchable(C2).status_register(S2)),
        Window::new(0x0C00, 0x0FFF, 1 * KIBIBYTE, Bank::RAM.switchable(C3).status_register(S3)),
        Window::new(0x1000, 0x13FF, 1 * KIBIBYTE, Bank::RAM.switchable(C4).status_register(S4)),
        Window::new(0x1400, 0x17FF, 1 * KIBIBYTE, Bank::RAM.switchable(C5).status_register(S5)),
        Window::new(0x1800, 0x1BFF, 1 * KIBIBYTE, Bank::RAM.switchable(C6).status_register(S6)),
        Window::new(0x1C00, 0x1FFF, 1 * KIBIBYTE, Bank::RAM.switchable(C7).status_register(S7)),
        Window::new(0x2000, 0x23FF, 1 * KIBIBYTE, Bank::RAM.switchable(C8).status_register(S8)),
        Window::new(0x2400, 0x27FF, 1 * KIBIBYTE, Bank::RAM.switchable(C9).status_register(S9)),
        Window::new(0x2800, 0x2BFF, 1 * KIBIBYTE, Bank::RAM.switchable(C10).status_register(S10)),
        Window::new(0x2C00, 0x2FFF, 1 * KIBIBYTE, Bank::RAM.switchable(C11).status_register(S11)),
    ])
    .ram_statuses(&[
        RamStatus::ReadOnly,
        RamStatus::ReadWrite,
    ])
    .build();

const READ_ONLY: u8 = 0;
const READ_WRITE: u8 = 1;

// Namco 129 and Namco 163
pub struct Mapper019 {
    // Actually a u15, but that's not ergonomic enough to use.
    irq_counter: u16,
    irq_pending: bool,

    allow_vram_in_lower_chr: bool,
    allow_vram_in_upper_chr: bool,
}

impl Mapper for Mapper019 {
    fn peek_cartridge_space(&self, params: &MapperParams, cpu_address: u16) -> ReadResult {
        match cpu_address {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x47FF => ReadResult::OPEN_BUS,
            0x4800..=0x4FFF => /* TODO: Expansion Audio */ ReadResult::full(0),
            0x5000..=0x57FF => ReadResult::full((self.irq_counter & 0b0000_0000_1111_1111) as u8),
            0x5800..=0x5FFF => ReadResult::full((self.irq_counter >> 8 & 0b0111_1111) as u8),
            0x6000..=0xFFFF => params.peek_prg(cpu_address),
        }
    }

    fn write_to_cartridge_space(&mut self, params: &mut MapperParams, cpu_address: u16, value: u8) {
        match cpu_address {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x47FF => { /* Do nothing. */ }
            0x4800..=0x4FFF => { /* TODO: Expansion Audio. */ }
            0x5000..=0x57FF => {
                self.irq_pending = false;
                // Set the low bits of the IRQ counter.
                self.irq_counter &= 0b0000_0000_1111_1111;
                self.irq_counter |= u16::from(value);
            }
            0x5800..=0x5FFF => {
                self.irq_pending = false;
                // Set the high bits of the IRQ counter.
                self.irq_counter &= 0b0111_1111_0000_0000;
                self.irq_counter |= u16::from(value << 1) << 7;
            }
            0x6000..=0x7FFF => { /* Do nothing. */ }
            0x8000..=0x87FF => set_chr_register(params, self.allow_vram_in_lower_chr, C0, S0, value),
            0x8800..=0x8FFF => set_chr_register(params, self.allow_vram_in_lower_chr, C1, S1, value),
            0x9000..=0x97FF => set_chr_register(params, self.allow_vram_in_lower_chr, C2, S2, value),
            0x9800..=0x9FFF => set_chr_register(params, self.allow_vram_in_lower_chr, C3, S3, value),
            0xA000..=0xA7FF => set_chr_register(params, self.allow_vram_in_upper_chr, C4, S4, value),
            0xA800..=0xAFFF => set_chr_register(params, self.allow_vram_in_upper_chr, C5, S5, value),
            0xB000..=0xB7FF => set_chr_register(params, self.allow_vram_in_upper_chr, C6, S6, value),
            0xB800..=0xBFFF => set_chr_register(params, self.allow_vram_in_upper_chr, C7, S7, value),
            0xC000..=0xC7FF => set_chr_register(params, true, C8, S8, value),
            0xC800..=0xCFFF => set_chr_register(params, true, C9, S9, value),
            0xD000..=0xD7FF => set_chr_register(params, true, C10, S10, value),
            0xD800..=0xDFFF => set_chr_register(params, true, C11, S11, value),
            0xE000..=0xE7FF => {
                // TODO: Pin 22 logic
                // TODO: Disable sound
                params.set_bank_register(P0, value & 0b0011_1111);
            }
            0xE800..=0xEFFF => {
                // TODO: NT CHR RAM
                params.set_bank_register(P1, value & 0b0011_1111);
            }
            0xF000..=0xF7FF => {
                // TODO: Pin 44 and PPU A12, A13
                params.set_bank_register(P2, value & 0b0011_1111);
            }
            0xF800..=0xFFFF => {
                let fields = splitbits!(min=u8, value, "ppppabcd");
                if fields.p == 0b0100 {
                    params.set_ram_status(S0, fields.a);
                    params.set_ram_status(S1, fields.b);
                    params.set_ram_status(S2, fields.c);
                    params.set_ram_status(S3, fields.d);
                } else {
                    // All read-only
                    params.set_ram_status(S0, 0);
                    params.set_ram_status(S1, 0);
                    params.set_ram_status(S2, 0);
                    params.set_ram_status(S3, 0);
                }
            }
        }
    }

    fn on_end_of_cpu_cycle(&mut self, _cycle: i64) {
        if self.irq_counter < 0x7FFF {
            self.irq_counter += 1;
            if self.irq_counter == 0x7FFF {
                self.irq_pending = true;
            }
        }
    }

    fn irq_pending(&self) -> bool {
        self.irq_pending
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}

fn set_chr_register(
    params: &mut MapperParams,
    allow_vram_in_chr: bool,
    reg_id: BankRegisterId,
    status_reg_id: RamStatusRegisterId,
    value: u8,
) {
    if allow_vram_in_chr && value >= 0xE0 {
        let vram_side = if value % 2 == 0 { VramSide::Left } else { VramSide::Right };
        params.set_bank_register_to_vram_side(reg_id, vram_side);
        params.set_ram_status(status_reg_id, READ_WRITE);
    } else {
        params.set_bank_register(reg_id, value);
        params.set_ram_status(status_reg_id, READ_ONLY);
    }
}

impl Mapper019 {
    pub fn new() -> Self {
        Self {
            irq_counter: 0,
            irq_pending: false,

            allow_vram_in_lower_chr: true,
            allow_vram_in_upper_chr: true,
        }
    }
}
