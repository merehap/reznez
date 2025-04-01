use crate::mapper::*;
use crate::memory::ppu::palette_ram::PaletteRam;
use crate::memory::ppu::ciram::{Ciram, CiramSide};

const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(512 * KIBIBYTE)
    .prg_layout(&[
        Window::new(0x6000, 0x67FF, 2 * KIBIBYTE, Bank::WORK_RAM.fixed_index(0).status_register(S12)),
        Window::new(0x6800, 0x6FFF, 2 * KIBIBYTE, Bank::WORK_RAM.fixed_index(1).status_register(S13)),
        Window::new(0x7000, 0x77FF, 2 * KIBIBYTE, Bank::WORK_RAM.fixed_index(2).status_register(S14)),
        Window::new(0x7800, 0x7FFF, 2 * KIBIBYTE, Bank::WORK_RAM.fixed_index(3).status_register(S15)),
        Window::new(0x8000, 0x9FFF, 8 * KIBIBYTE, Bank::ROM.switchable(P0)),
        Window::new(0xA000, 0xBFFF, 8 * KIBIBYTE, Bank::ROM.switchable(P1)),
        Window::new(0xC000, 0xDFFF, 8 * KIBIBYTE, Bank::ROM.switchable(P2)),
        Window::new(0xE000, 0xFFFF, 8 * KIBIBYTE, Bank::ROM.fixed_index(-1)),
    ])
    .chr_rom_max_size(256 * KIBIBYTE)
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
    .read_write_statuses(&[
        ReadWriteStatus::ReadOnly,
        ReadWriteStatus::ReadWrite,
    ])
    .build();

const READ_ONLY: u8 = 0;
const READ_WRITE: u8 = 1;

// Namco 129 and Namco 163
pub struct Mapper019 {
    // Actually a u15, but that's not ergonomic enough to use.
    irq_counter: u16,

    allow_ciram_in_low_chr: bool,
    allow_ciram_in_high_chr: bool,
}

impl Mapper for Mapper019 {
    fn peek_cartridge_space(&self, params: &MapperParams, cpu_address: u16) -> ReadResult {
        match cpu_address {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x47FF => ReadResult::OPEN_BUS,
            0x4800..=0x4FFF => /* TODO: Expansion Audio */ ReadResult::full(0),
            0x5000..=0x57FF => ReadResult::full((self.irq_counter & 0b0000_0000_1111_1111) as u8),
            0x5800..=0x5FFF => ReadResult::full(((self.irq_counter >> 8) & 0b0111_1111) as u8),
            0x6000..=0xFFFF => params.peek_prg(cpu_address),
        }
    }

    fn write_to_cartridge_space(&mut self, params: &mut MapperParams, cpu_address: u16, value: u8) {
        match cpu_address {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x47FF => { /* Do nothing. */ }
            0x4800..=0x4FFF => { /* TODO: Expansion Audio. */ }
            0x5000..=0x57FF => {
                params.set_irq_pending(false);
                // Set the low bits of the IRQ counter.
                self.irq_counter &= 0b0000_0000_1111_1111;
                self.irq_counter |= u16::from(value);
            }
            0x5800..=0x5FFF => {
                params.set_irq_pending(false);
                // Set the high bits of the IRQ counter.
                self.irq_counter &= 0b0111_1111_0000_0000;
                self.irq_counter |= u16::from(value << 1) << 7;
            }
            0x6000..=0x7FFF => { /* Do nothing. */ }
            0x8000..=0x87FF => set_chr_register(params, self.allow_ciram_in_low_chr, C0, S0, value),
            0x8800..=0x8FFF => set_chr_register(params, self.allow_ciram_in_low_chr, C1, S1, value),
            0x9000..=0x97FF => set_chr_register(params, self.allow_ciram_in_low_chr, C2, S2, value),
            0x9800..=0x9FFF => set_chr_register(params, self.allow_ciram_in_low_chr, C3, S3, value),
            0xA000..=0xA7FF => set_chr_register(params, self.allow_ciram_in_high_chr, C4, S4, value),
            0xA800..=0xAFFF => set_chr_register(params, self.allow_ciram_in_high_chr, C5, S5, value),
            0xB000..=0xB7FF => set_chr_register(params, self.allow_ciram_in_high_chr, C6, S6, value),
            0xB800..=0xBFFF => set_chr_register(params, self.allow_ciram_in_high_chr, C7, S7, value),
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
                let fields = splitbits!(value, "hlpp pppp");
                self.allow_ciram_in_high_chr = !fields.h;
                self.allow_ciram_in_low_chr = !fields.l;
                params.set_bank_register(P1, fields.p);
            }
            0xF000..=0xF7FF => {
                // TODO: Pin 44 and PPU A12, A13
                params.set_bank_register(P2, value & 0b0011_1111);
            }
            0xF800..=0xFFFF => {
                let fields = splitbits!(min=u8, value, "ppppabcd");
                if fields.p == 0b0100 {
                    params.set_read_write_status(S0, fields.a);
                    params.set_read_write_status(S1, fields.b);
                    params.set_read_write_status(S2, fields.c);
                    params.set_read_write_status(S3, fields.d);
                } else {
                    // All read-only
                    params.set_read_write_status(S0, 0);
                    params.set_read_write_status(S1, 0);
                    params.set_read_write_status(S2, 0);
                    params.set_read_write_status(S3, 0);
                }
            }
        }
    }

    fn ppu_peek(
        &self,
        params: &MapperParams,
        ciram: &Ciram,
        palette_ram: &PaletteRam,
        mut address: PpuAddress,
    ) -> u8 {
        match address.to_u16() {
            0x0000..=0x3EFF => {
                if address.to_u16() >= 0x3000 {
                    // Mirror down, just like normal ppu_peek.
                    address = PpuAddress::from_u16(address.to_u16() - 0x1000);
                }

                params.peek_chr(ciram, address)
            }
            0x3F00..=0x3FFF => self.peek_palette_table_byte(palette_ram, address),
            0x4000..=0xFFFF => unreachable!(),
        }
    }

    #[inline]
    fn ppu_write(
        &mut self,
        params: &mut MapperParams,
        ciram: &mut Ciram,
        palette_ram: &mut PaletteRam,
        mut address: PpuAddress,
        value: u8,
    ) {
        match address.to_u16() {
            0x0000..=0x3EFF => {
                if address.to_u16() >= 0x3000 {
                    // Mirror down, just like normal ppu_write.
                    address = PpuAddress::from_u16(address.to_u16() - 0x1000);
                }

                params.write_chr(ciram, address, value);
            }
            0x3F00..=0x3FFF => self.write_palette_table_byte(palette_ram, address, value),
            0x4000..=0xFFFF => unreachable!(),
        }
    }

    fn on_end_of_cpu_cycle(&mut self, params: &mut MapperParams, _cycle: i64) {
        if self.irq_counter < 0x7FFF {
            self.irq_counter += 1;
            if self.irq_counter == 0x7FFF {
                params.set_irq_pending(true);
            }
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}

fn set_chr_register(
    params: &mut MapperParams,
    allow_ciram_in_chr: bool,
    reg_id: BankRegisterId,
    status_reg_id: ReadWriteStatusRegisterId,
    value: u8,
) {
    if allow_ciram_in_chr && value >= 0xE0 {
        let ciram_side = if value & 1 == 0 { CiramSide::Left } else { CiramSide::Right };
        params.set_bank_register_to_ciram_side(reg_id, ciram_side);
        params.set_read_write_status(status_reg_id, READ_WRITE);
    } else {
        params.set_bank_register(reg_id, value);
        params.set_read_write_status(status_reg_id, READ_ONLY);
    }
}

impl Mapper019 {
    pub fn new() -> Self {
        Self {
            irq_counter: 0,

            allow_ciram_in_low_chr: true,
            allow_ciram_in_high_chr: true,
        }
    }
}
