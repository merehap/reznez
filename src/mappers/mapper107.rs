use crate::mapper::*;

const LAYOUT: Layout = Layout::builder()
    // Oversize. Actual cartridge only has 128 max.
    .prg_rom_max_size(4096 * KIBIBYTE)
    .prg_layout(&[
        Window::new(0x6000, 0x7FFF,  8 * KIBIBYTE, Bank::EMPTY),
        Window::new(0x8000, 0xFFFF, 32 * KIBIBYTE, Bank::ROM.switchable(P0)),
    ])
    // Oversize. Actual cartridge only has 64 max.
    .chr_rom_max_size(2048 * KIBIBYTE)
    .chr_layout(&[
        Window::new(0x0000, 0x1FFF, 8 * KIBIBYTE, Bank::ROM.switchable(C0)),
    ])
    .build();

// Magic Dragon 
pub struct Mapper107;

impl Mapper for Mapper107 {
    fn write_to_cartridge_space(&mut self, params: &mut MapperParams, cpu_address: u16, value: u8) {
        match cpu_address {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x7FFF => { /* Do nothing. */ }
            0x8000..=0xFFFF => {
                // The PRG and CHR registers overlap.
                params.set_bank_register(P0, value >> 1);
                params.set_bank_register(C0, value);
            }
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}
