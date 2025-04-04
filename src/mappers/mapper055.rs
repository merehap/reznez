use crate::mapper::*;

const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(64 * KIBIBYTE)
    .prg_bank_size_override(16 * KIBIBYTE)
    .prg_layout(&[
        Window::new(0x6000, 0x67FF,  2 * KIBIBYTE, Bank::ROM.fixed_index(2)),
        Window::new(0x6800, 0x6FFF,  2 * KIBIBYTE, Bank::MirrorOf(0x6000)),
        Window::new(0x7000, 0x77FF,  2 * KIBIBYTE, Bank::ROM.fixed_index(3)),
        Window::new(0x7800, 0x7FFF,  2 * KIBIBYTE, Bank::MirrorOf(0x7000)),
        // These two could be a single 32KiB bank, but the bank indexes are clearer this way.
        Window::new(0x8000, 0xBFFF, 16 * KIBIBYTE, Bank::ROM.fixed_index(0)),
        Window::new(0xC000, 0xFFFF, 16 * KIBIBYTE, Bank::ROM.fixed_index(1)),
    ])
    .chr_rom_max_size(8 * KIBIBYTE)
    .chr_layout(&[
        Window::new(0x0000, 0x1FFF, 8 * KIBIBYTE, Bank::ROM.fixed_index(0)),
    ])
    .build();

// BTL-MARIO1-MALEE2
pub struct Mapper055;

impl Mapper for Mapper055 {
    fn write_to_cartridge_space(&mut self, _params: &mut MapperParams, cpu_address: u16, _value: u8) {
        match cpu_address {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0xFFFF => { /* Do nothing here, just like NROM. */ }
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}
