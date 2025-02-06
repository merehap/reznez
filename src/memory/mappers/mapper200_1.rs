use crate::memory::mapper::*;

const LAYOUT: Layout = Layout::builder()
    .prg_max_size(128 * KIBIBYTE)
    .prg_layout(&[
        Window::new(0x6000, 0x7FFF,  8 * KIBIBYTE, Bank::EMPTY),
        Window::new(0x8000, 0xBFFF, 16 * KIBIBYTE, Bank::ROM.switchable(P0)),
        Window::new(0xC000, 0xFFFF, 16 * KIBIBYTE, Bank::mirror_of(0x8000)),
    ])
    .chr_max_size(64 * KIBIBYTE)
    .chr_layout(&[
        Window::new(0x0000, 0x1FFF, 8 * KIBIBYTE, Bank::ROM.switchable(C0)),
    ])
    .name_table_mirrorings(&[
        NameTableMirroring::Vertical,
        NameTableMirroring::Horizontal,
    ])
    .build();

// NROM-128 multicarts with 8 PRG/CHR banks
pub struct Mapper200_1;

impl Mapper for Mapper200_1 {
    fn write_to_cartridge_space(&mut self, params: &mut MapperParams, cpu_address: u16, _value: u8) {
        match cpu_address {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x7FFF => { /* Do nothing. */ }
            0x8000..=0xFFFF => {
                let bank_index = cpu_address & 0x0007;
                params.set_bank_register(P0, bank_index);
                params.set_bank_register(C0, bank_index);
                params.set_name_table_mirroring((cpu_address >> 2) as u8 & 1);
            }
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}
