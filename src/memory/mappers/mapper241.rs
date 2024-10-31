use crate::memory::mapper::*;

const LAYOUT: Layout = Layout::builder()
    .prg_max_size(8192 * KIBIBYTE)
    .chr_max_size(8 * KIBIBYTE)
    .prg_layout(&[
        Window::new(0x6000, 0x7FFF,  8 * KIBIBYTE, Bank::WORK_RAM),
        Window::new(0x8000, 0xFFFF, 32 * KIBIBYTE, Bank::ROM.switchable(P0)),
    ])
    .chr_layout(&[
        Window::new(0x0000, 0x1FFF, 8 * KIBIBYTE, Bank::RAM.fixed_index(0)),
    ])
    .build();

// BxROM with WorkRam
pub struct Mapper241;

impl Mapper for Mapper241 {
    fn write_to_cartridge_space(&mut self, params: &mut MapperParams, address: CpuAddress, value: u8) {
        match address.to_raw() {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x7FFF => { /* Do nothing. */ }
            0x8000..=0xFFFF => params.set_bank_register(P0, value),
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}
