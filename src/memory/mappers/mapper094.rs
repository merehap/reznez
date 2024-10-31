use crate::memory::mapper::*;

const LAYOUT: Layout = Layout::builder()
    .prg_max_size(128 * KIBIBYTE)
    .chr_max_size(8 * KIBIBYTE)
    .prg_layout(&[
        Window::new(0x6000, 0x7FFF,  8 * KIBIBYTE, Bank::EMPTY),
        Window::new(0x8000, 0xBFFF, 16 * KIBIBYTE, Bank::ROM.switchable(P0)),
        Window::new(0xC000, 0xFFFF, 16 * KIBIBYTE, Bank::ROM.fixed_index(BankIndex::LAST)),
    ])
    .chr_layout(&[
        Window::new(0x0000, 0x1FFF, 8 * KIBIBYTE, Bank::ROM.fixed_index(BankIndex::FIRST)),
    ])
    .build();

// UxROM, but the register is shifted by two bits.
pub struct Mapper094;

impl Mapper for Mapper094 {
    fn write_to_cartridge_space(&mut self, params: &mut MapperParams, address: CpuAddress, value: u8) {
        match address.to_raw() {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x7FFF => { /* Do nothing. */ }
            0x8000..=0xFFFF => params.set_bank_register(P0, splitbits_named!(value, "...ppp..")),
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}

