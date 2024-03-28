use crate::memory::mapper::*;

const PRG_LAYOUT: PrgLayout = PrgLayout::new(&[
    PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::Empty),
    PrgWindow::new(0x8000, 0xFFFF, 32 * KIBIBYTE, PrgBank::Switchable(Rom, P0)),
]);

const CHR_LAYOUT: ChrLayout = ChrLayout::new(&[
    ChrWindow::new(0x0000, 0x1FFF, 8 * KIBIBYTE, ChrBank::Switchable(Rom, C0)),
]);

// NTD-8 (extended PRG and CHR from NINA-03 and NINA-06)
pub struct Mapper113;

impl Mapper for Mapper113 {
    fn initial_layout(&self) -> InitialLayout {
        InitialLayout::builder()
            .prg_max_bank_count(8)
            .prg_bank_size(32 * KIBIBYTE)
            .prg_windows(PRG_LAYOUT)
            .chr_max_bank_count(16)
            .chr_bank_size(8 * KIBIBYTE)
            .chr_windows(CHR_LAYOUT)
            .name_table_mirroring_source(NameTableMirroringSource::Cartridge)
            .build()
    }

    fn write_to_cartridge_space(&mut self, params: &mut MapperParams, address: CpuAddress, value: u8) {
        let address = address.to_raw();
        match address {
            0x0000..=0x401F => unreachable!(),
            // 0x41XX, 0x43XX, ... $5DXX, $5FXX
            0x4100..=0x5FFF if (address / 0x100) % 2 == 1 => {
                let vertical =  value & 0b1000_0000 != 0;
                let high_chr = (value & 0b0100_0000) >> 3;
                let prg      = (value & 0b0011_1000) >> 3;
                let low_chr  =  value & 0b0000_0111;

                let mirroring = if vertical {
                    NameTableMirroring::Vertical
                } else {
                    NameTableMirroring::Horizontal
                };

                params.set_name_table_mirroring(mirroring);
                params.set_bank_index_register(C0, high_chr | low_chr);
                params.set_bank_index_register(P0, prg);
            }
            _ => { /* Do nothing. */ }
        }
    }
}
