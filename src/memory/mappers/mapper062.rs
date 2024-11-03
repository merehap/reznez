use crate::memory::mapper::*;

const LAYOUT: Layout = Layout::builder()
    .prg_max_size(4096 * KIBIBYTE)
    .prg_layout(&[
        Window::new(0x6000, 0x7FFF,  8 * KIBIBYTE, Bank::EMPTY),
        Window::new(0x8000, 0xFFFF, 32 * KIBIBYTE, Bank::ROM.switchable(P0)),
    ])
    .prg_layout(&[
        Window::new(0x6000, 0x7FFF,  8 * KIBIBYTE, Bank::EMPTY),
        Window::new(0x8000, 0xBFFF, 16 * KIBIBYTE, Bank::ROM.switchable(P0)),
        Window::new(0xC000, 0xFFFF, 16 * KIBIBYTE, Bank::ROM.switchable(P0)),
    ])
    .chr_max_size(1024 * KIBIBYTE)
    .chr_layout(&[
        Window::new(0x0000, 0x1FFF, 8 * KIBIBYTE, Bank::ROM.switchable(C0)),
    ])
    .name_table_mirrorings(&[
        NameTableMirroring::Vertical,
        NameTableMirroring::Horizontal,
    ])
    .build();

// Super 700-in-1
pub struct Mapper062; 

impl Mapper for Mapper062 {
    fn write_to_cartridge_space(&mut self, params: &mut MapperParams, address: CpuAddress, value: u8) {
        match address.to_raw() {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x7FFF => { /* Do nothing. */ }
            0x8000..=0xFFFF => {
                let fields = splitbits!(min=u8, address.to_raw(), "..pp pppp mqlc cccc");
                params.set_name_table_mirroring(fields.m);
                params.set_prg_layout(fields.l);

                let prg_index = combinebits!(fields.q, fields.p, "0qpppppp");
                params.set_bank_register(P0, prg_index);
                let chr_index = (fields.c << 2) | (value & 0b11);
                params.set_bank_register(C0, chr_index);
            }
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}
