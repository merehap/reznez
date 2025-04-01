use crate::mapper::*;

const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(2048 * KIBIBYTE)
    .prg_layout(&[
        Window::new(0x6000, 0x7EFF, 7 * KIBIBYTE + 3 * KIBIBYTE / 4, Bank::EMPTY),
        Window::new(0x7F00, 0x7F7F, KIBIBYTE / 8, Bank::WORK_RAM.status_register(S0)),
        Window::new(0x7F80, 0x7FFF, KIBIBYTE / 8, Bank::MirrorOf(0x7F00)),
        Window::new(0x8000, 0x9FFF, 8 * KIBIBYTE, Bank::ROM.switchable(P0)),
        Window::new(0xA000, 0xBFFF, 8 * KIBIBYTE, Bank::ROM.switchable(P1)),
        Window::new(0xC000, 0xDFFF, 8 * KIBIBYTE, Bank::ROM.switchable(P2)),
        Window::new(0xE000, 0xFFFF, 8 * KIBIBYTE, Bank::ROM.fixed_index(-1)),
    ])
    .chr_rom_max_size(256 * KIBIBYTE)
    .chr_layout(&[
        Window::new(0x0000, 0x07FF, 2 * KIBIBYTE, Bank::ROM.switchable(C0)),
        Window::new(0x0800, 0x0FFF, 2 * KIBIBYTE, Bank::ROM.switchable(C1)),
        Window::new(0x1000, 0x13FF, 1 * KIBIBYTE, Bank::ROM.switchable(C2)),
        Window::new(0x1400, 0x17FF, 1 * KIBIBYTE, Bank::ROM.switchable(C3)),
        Window::new(0x1800, 0x1BFF, 1 * KIBIBYTE, Bank::ROM.switchable(C4)),
        Window::new(0x1C00, 0x1FFF, 1 * KIBIBYTE, Bank::ROM.switchable(C5)),
    ])
    .name_table_mirrorings(&[
        NameTableMirroring::HORIZONTAL,
        NameTableMirroring::VERTICAL,
    ])
    .read_write_statuses(&[
        ReadWriteStatus::Disabled,
        ReadWriteStatus::ReadWrite,
    ])
    .build();

// Taito's X1-005
pub struct Mapper080;

impl Mapper for Mapper080 {
    fn write_to_cartridge_space(&mut self, params: &mut MapperParams, cpu_address: u16, value: u8) {
        match cpu_address {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x7EEF => { /* Do nothing. */ }
            0x7EF0 => params.set_bank_register(C0, value),
            0x7EF1 => params.set_bank_register(C1, value),
            0x7EF2 => params.set_bank_register(C2, value),
            0x7EF3 => params.set_bank_register(C3, value),
            0x7EF4 => params.set_bank_register(C4, value),
            0x7EF5 => params.set_bank_register(C5, value),
            0x7EF6..=0x7EF7 => params.set_name_table_mirroring(value & 1),
            0x7EF8..=0x7EF9 => {
                let ram_enabled = value == 0xA3;
                params.set_read_write_status(S0, ram_enabled as u8);
            }
            0x7EFA..=0x7EFB => params.set_bank_register(P0, value),
            0x7EFC..=0x7EFD => params.set_bank_register(P1, value),
            0x7EFE..=0x7EFF => params.set_bank_register(P2, value),
            0x7F00..=0xFFFF => { /* Do nothing. */ }
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}
