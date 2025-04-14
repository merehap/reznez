use crate::mapper::*;

const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(256 * KIBIBYTE)
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::EMPTY),
        PrgWindow::new(0x8000, 0xFFFF, 32 * KIBIBYTE, PrgBank::ROM.switchable(P0)),
    ])
    .chr_rom_max_size(128 * KIBIBYTE)
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x1FFF, 8 * KIBIBYTE, ChrBank::ROM.switchable(C0)),
    ])
    .name_table_mirrorings(&[
        NameTableMirroring::VERTICAL,
        NameTableMirroring::HORIZONTAL,
    ])
    .build();

// Caltron 6-in-1
// FIXME: Doesn't work. CHR bank switching may be broken.
#[derive(Default)]
pub struct Mapper041 {
    inner_bank_select_enabled: bool,
}

impl Mapper for Mapper041 {
    fn write_to_cartridge_space(&mut self, params: &mut MapperParams, cpu_address: u16, value: u8) {
        match cpu_address {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x5FFF => { /* Do nothing. */ }
            0x6000..=0x67FF => {
                let fields = splitbits!(value, "........ ..mccppp");
                params.set_name_table_mirroring(fields.m as u8);
                params.set_chr_bank_register_bits(C0, (fields.c << 2).into(), 0b0000_1100);
                self.inner_bank_select_enabled = fields.p & 0b100 != 0;
                params.set_prg_register(P0, fields.p);
            }
            0x6800..=0x7FFF => { /* Do nothing. */ }
            0x8000..=0xFFFF => {
                if self.inner_bank_select_enabled {
                    params.set_chr_bank_register_bits(C0, value.into(), 0b0000_0011);
                }
            }
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}
