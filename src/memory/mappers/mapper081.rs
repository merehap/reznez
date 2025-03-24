use crate::memory::mapper::*;

const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(64 * KIBIBYTE)
    .prg_layout(&[
        Window::new(0x6000, 0x7FFF,  8 * KIBIBYTE, Bank::EMPTY),
        Window::new(0x8000, 0xBFFF, 16 * KIBIBYTE, Bank::ROM.switchable(P0)),
        Window::new(0xC000, 0xFFFF, 16 * KIBIBYTE, Bank::ROM.fixed_index(-1)),
    ])
    .chr_rom_max_size(32 * KIBIBYTE)
    .chr_layout(&[
        Window::new(0x0000, 0x1FFF, 8 * KIBIBYTE, Bank::ROM.switchable(C0)),
    ])
    .build();

// NTDEC N715021 (Super Gun)
// TODO: Untested. Need test ROM.
pub struct Mapper081;

impl Mapper for Mapper081 {
    fn write_to_cartridge_space(&mut self, params: &mut MapperParams, cpu_address: u16, _value: u8) {
        match cpu_address {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x7FFF => { /* Do nothing. */ }
            0x8000..=0xFFFF => {
                let fields = splitbits!(cpu_address, ".... .... .... ppcc");
                params.set_bank_register(P0, fields.p);
                params.set_bank_register(C0, fields.c);
            }
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}
