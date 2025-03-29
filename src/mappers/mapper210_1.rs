use crate::mapper::*;

const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(512 * KIBIBYTE)
    .prg_layout(&[
        // FIXME: This is supposed to be 2KiBs mirrored. Family Circuit doesn't work without it.
        Window::new(0x6000, 0x7FFF, 8 * KIBIBYTE, Bank::WORK_RAM),
        Window::new(0x8000, 0x9FFF, 8 * KIBIBYTE, Bank::ROM.switchable(P0)),
        Window::new(0xA000, 0xBFFF, 8 * KIBIBYTE, Bank::ROM.switchable(P1)),
        Window::new(0xC000, 0xDFFF, 8 * KIBIBYTE, Bank::ROM.switchable(P2)),
        Window::new(0xE000, 0xFFFF, 8 * KIBIBYTE, Bank::ROM.fixed_index(-1)),
    ])
    .chr_rom_max_size(256 * KIBIBYTE)
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
    .build();

// Namco 175
pub struct Mapper210_1;

impl Mapper for Mapper210_1 {
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
            0xC000..=0xC7FF => { /* TODO: External PRG RAM enable. */ }
            0xC800..=0xDFFF => { /* Do nothing. */ }
            0xE000..=0xE7FF => params.set_bank_register(P0, value & 0b0011_1111),
            0xE800..=0xEFFF => params.set_bank_register(P1, value & 0b0011_1111),
            0xF000..=0xF7FF => params.set_bank_register(P2, value & 0b0011_1111),
            0xF800..=0xFFFF => { /* Do nothing. */ }
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}
