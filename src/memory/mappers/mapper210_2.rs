use crate::memory::mapper::*;

const LAYOUT: Layout = Layout::builder()
    .prg_max_size(512 * KIBIBYTE)
    .prg_layout(&[
        Window::new(0x6000, 0x7FFF, 8 * KIBIBYTE, Bank::EMPTY),
        Window::new(0x8000, 0x9FFF, 8 * KIBIBYTE, Bank::ROM.switchable(P0)),
        Window::new(0xA000, 0xBFFF, 8 * KIBIBYTE, Bank::ROM.switchable(P1)),
        Window::new(0xC000, 0xDFFF, 8 * KIBIBYTE, Bank::ROM.switchable(P2)),
        Window::new(0xE000, 0xFFFF, 8 * KIBIBYTE, Bank::ROM.fixed_index(-1)),
    ])
    .chr_max_size(256 * KIBIBYTE)
    .chr_layout(&[
        Window::new(0x0000, 0x03FF, 1 * KIBIBYTE, Bank::ROM.switchable(C0)),
        Window::new(0x0400, 0x07FF, 1 * KIBIBYTE, Bank::ROM.switchable(C1)),
        Window::new(0x0800, 0x0BFF, 1 * KIBIBYTE, Bank::ROM.switchable(C2)),
        Window::new(0x0C00, 0x0FFF, 1 * KIBIBYTE, Bank::ROM.switchable(C3)),
        Window::new(0x1000, 0x13FF, 1 * KIBIBYTE, Bank::ROM.switchable(C4)),
        Window::new(0x1400, 0x17FF, 1 * KIBIBYTE, Bank::ROM.switchable(C5)),
        Window::new(0x1800, 0x1BFF, 1 * KIBIBYTE, Bank::ROM.switchable(C6)),
        Window::new(0x1C00, 0x1FFF, 1 * KIBIBYTE, Bank::ROM.switchable(C7)),
    ])
    .name_table_mirrorings(&[
        NameTableMirroring::OneScreenLeftBank,
        NameTableMirroring::Vertical,
        NameTableMirroring::OneScreenRightBank,
        NameTableMirroring::Horizontal,
    ])
    .build();

// Namco 340
// TODO: Untested! Need relevant ROMs to test against (everything is mapper 19 instead).
pub struct Mapper210_2;

impl Mapper for Mapper210_2 {
    fn write_to_cartridge_space(&mut self, params: &mut MapperParams, cpu_address: u16, value: u8) {
        match cpu_address {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x7FFF => { /* Do nothing. */ }
            0x8000..=0x87FF => params.set_bank_register(C0, value),
            0x8800..=0x8FFF => params.set_bank_register(C1, value),
            0x9000..=0x97FF => params.set_bank_register(C2, value),
            0x9800..=0x9FFF => params.set_bank_register(C3, value),
            0xA000..=0xA7FF => params.set_bank_register(C4, value),
            0xA800..=0xAFFF => params.set_bank_register(C5, value),
            0xB000..=0xB7FF => params.set_bank_register(C6, value),
            0xB800..=0xBFFF => params.set_bank_register(C7, value),
            0xC000..=0xDFFF => { /* Do nothing. */ }
            0xE000..=0xE7FF => {
                let fields = splitbits!(min=u8, value, "mmpppppp");
                params.set_name_table_mirroring(fields.m);
                params.set_bank_register(P0, fields.p);
            }
            0xE800..=0xEFFF => params.set_bank_register(P1, value & 0b0011_1111),
            0xF000..=0xF7FF => params.set_bank_register(P2, value & 0b0011_1111),
            0xF800..=0xFFFF => { /* Do nothing. */ }
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}
