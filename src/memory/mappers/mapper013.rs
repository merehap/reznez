use crate::memory::mapper::*;

const LAYOUT: Layout = Layout::builder()
    .prg_max_size(32 * KIBIBYTE)
    .chr_max_size(16 * KIBIBYTE)
    .prg_layout(&[
        Window::new(0x6000, 0x7FFF,  8 * KIBIBYTE, Bank::EMPTY),
        Window::new(0x8000, 0xFFFF, 32 * KIBIBYTE, Bank::ROM.fixed_index(0)),
    ])
    .chr_layout(&[
        Window::new(0x0000, 0x0FFF, 4 * KIBIBYTE, Bank::RAM.fixed_index(0)),
        Window::new(0x1000, 0x1FFF, 4 * KIBIBYTE, Bank::RAM.switchable(C0)),
    ])
    .build();

// CPROM
pub struct Mapper013;

impl Mapper for Mapper013 {
    fn write_to_cartridge_space(&mut self, params: &mut MapperParams, cpu_address: CpuAddress, value: u8) {
        match cpu_address.to_raw() {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x7FFF => { /* Do nothing. */ },
            0x8000..=0xFFFF => params.set_bank_register(C0, value & 0b11),
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}
