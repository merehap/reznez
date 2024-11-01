use crate::memory::mapper::*;

const LAYOUT: Layout = Layout::builder()
    .prg_max_size(256 * KIBIBYTE)
    .prg_layout(&[
        Window::new(0x6000, 0x7FFF,  8 * KIBIBYTE, Bank::EMPTY),
        Window::new(0x8000, 0xFFFF, 32 * KIBIBYTE, Bank::ROM.switchable(P0)),
    ])
    .chr_max_size(128 * KIBIBYTE)
    .chr_layout(&[
        Window::new(0x0000, 0x1FFF, 8 * KIBIBYTE, Bank::ROM.switchable(C0)),
    ])
    .build();

const MIRRORINGS: [NameTableMirroring; 2] = [
    NameTableMirroring::Horizontal,
    NameTableMirroring::Vertical,
];

// NTD-8 (extended PRG and CHR from NINA-03 and NINA-06)
pub struct Mapper113;

impl Mapper for Mapper113 {
    fn write_to_cartridge_space(&mut self, params: &mut MapperParams, address: CpuAddress, value: u8) {
        let address = address.to_raw();
        match address {
            0x0000..=0x401F => unreachable!(),
            // 0x41XX, 0x43XX, ... $5DXX, $5FXX
            0x4100..=0x5FFF if (address / 0x100) % 2 == 1 => {
                let fields = splitbits!(value, "mcpppccc");
                params.set_name_table_mirroring(MIRRORINGS[fields.m as usize]);
                params.set_bank_register(C0, fields.c);
                params.set_bank_register(P0, fields.p);
            }
            _ => { /* Do nothing. */ }
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}
