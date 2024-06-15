use crate::memory::mapper::*;

const PRG_LAYOUT: PrgLayout = PrgLayout::new(&[
    PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, Bank::EMPTY),
    PrgWindow::new(0x8000, 0xFFFF, 32 * KIBIBYTE, Bank::switchable_rom(P0)),
]);

const CHR_LAYOUT: ChrLayout = ChrLayout::new(&[
    ChrWindow::new(0x0000, 0x1FFF, 8 * KIBIBYTE, Bank::switchable_rom(C0)),
]);

const MIRRORINGS: [NameTableMirroring; 2] = [
    NameTableMirroring::Horizontal,
    NameTableMirroring::Vertical,
];

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
                let fields = splitbits!(value, "mcpppccc");
                params.set_name_table_mirroring(MIRRORINGS[fields.m as usize]);
                params.set_bank_register(C0, fields.c);
                params.set_bank_register(P0, fields.p);
            }
            _ => { /* Do nothing. */ }
        }
    }
}
