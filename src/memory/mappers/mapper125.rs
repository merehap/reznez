use crate::memory::mapper::*;

const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(4096 * KIBIBYTE)
    .prg_layout(&[
        Window::new(0x6000, 0x7FFF, 8 * KIBIBYTE, Bank::ROM.switchable(P0)),
        Window::new(0x8000, 0x9FFF, 8 * KIBIBYTE, Bank::ROM.fixed_index(12)),
        Window::new(0xA000, 0xBFFF, 8 * KIBIBYTE, Bank::ROM.fixed_index(13)),
        Window::new(0xC000, 0xDFFF, 8 * KIBIBYTE, Bank::WORK_RAM),
        Window::new(0xE000, 0xFFFF, 8 * KIBIBYTE, Bank::ROM.fixed_index(15)),
    ])
    .chr_rom_max_size(8 * KIBIBYTE)
    .chr_layout(&[
        Window::new(0x0000, 0x1FFF, 8 * KIBIBYTE, Bank::RAM.fixed_index(0)),
    ])
    .build();

// Monty on the Run (Whirlwind Manu's FDS conversion)
// FIXME: Untested due to lack of test ROM
pub struct Mapper125;

impl Mapper for Mapper125 {
    fn write_to_cartridge_space(&mut self, params: &mut MapperParams, cpu_address: u16, value: u8) {
        match cpu_address {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x5FFF => { /* Do nothing. */ }
            0x6000..=0x7FFF => params.set_bank_register(P0, value & 0b1111),
            0x8000..=0xFFFF => { /* Do nothing. */ }
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}
