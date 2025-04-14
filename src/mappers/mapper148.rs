use crate::mapper::*;

const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(64 * KIBIBYTE)
    .prg_layout(&[
        Window::new(0x6000, 0x7FFF,  8 * KIBIBYTE, Bank::EMPTY),
        Window::new(0x8000, 0xFFFF, 32 * KIBIBYTE, Bank::ROM.switchable(P0)),
    ])
    .chr_rom_max_size(64 * KIBIBYTE)
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x1FFF, 8 * KIBIBYTE, ChrBank::ROM.switchable(C0)),
    ])
    .build();

// Sachen SA-008-A and Tengen 800008
pub struct Mapper148;

impl Mapper for Mapper148 {
    fn write_to_cartridge_space(&mut self, params: &mut MapperParams, cpu_address: u16, value: u8) {
        match cpu_address {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x7FFF => { /* Do nothing. */ }
            0x8000..=0xFFFF => {
                let banks = splitbits!(value, "....pccc");
                params.set_bank_register(P0, banks.p);
                params.set_chr_register(C0, banks.c);
            }
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}
