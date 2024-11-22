use crate::memory::mapper::*;

const LAYOUT: Layout = Layout::builder()
    .prg_max_size(32 * KIBIBYTE)
    .prg_layout(&[
        Window::new(0x6000, 0x7FFF,  8 * KIBIBYTE, Bank::EMPTY),
        Window::new(0x8000, 0xFFFF, 32 * KIBIBYTE, Bank::ROM.fixed_index(0)),
    ])
    .chr_max_size(64 * KIBIBYTE)
    .chr_layout(&[
        Window::new(0x0000, 0x0FFF, 4 * KIBIBYTE, Bank::ROM.switchable(C0)),
        Window::new(0x1000, 0x1FFF, 4 * KIBIBYTE, Bank::ROM.switchable(C1)),
    ])
    .build();

// Sunsoft-1
pub struct Mapper184;

impl Mapper for Mapper184 {
    fn write_to_cartridge_space(&mut self, params: &mut MapperParams, cpu_address: u16, value: u8) {
        match cpu_address {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x5FFF => { /* Do nothing. */ }
            0x6000..=0x7FFF => {
                let (high_bank, low_bank) = splitbits_named!(value, ".hhh.lll");
                params.set_bank_register(C0, low_bank);
                params.set_bank_register(C1, high_bank);
            }
            0x8000..=0xFFFF => { /* Do nothing. */ }
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}
