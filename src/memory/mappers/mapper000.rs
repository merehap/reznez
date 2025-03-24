use crate::memory::mapper::*;

const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(32 * KIBIBYTE)
    .prg_layout(&[
        Window::new(0x6000, 0x7FFF,  8 * KIBIBYTE, Bank::EMPTY),
        Window::new(0x8000, 0xFFFF, 32 * KIBIBYTE, Bank::ROM.fixed_index(0)),
    ])
    .chr_rom_max_size(8 * KIBIBYTE)
    .chr_layout(&[
        Window::new(0x0000, 0x1FFF, 8 * KIBIBYTE, Bank::ROM.fixed_index(0)),
    ])
    .build();

// NROM
// The simplest mapper. Defined entirely by its Layout, it doesn't actually have any custom logic.
pub struct Mapper000;

impl Mapper for Mapper000 {
    fn write_to_cartridge_space(&mut self, _params: &mut MapperParams, cpu_address: u16, _value: u8) {
        match cpu_address {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0xFFFF => { /* Only mapper 0 does nothing here. */ }
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}
