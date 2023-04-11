use crate::memory::mapper::*;

const PRG_LAYOUT: PrgLayout = PrgLayout::new(&[
    PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgType::Empty),
    PrgWindow::new(0x8000, 0xBFFF, 16 * KIBIBYTE, PrgType::SwitchableBank(Rom, P0)),
    PrgWindow::new(0xC000, 0xFFFF, 16 * KIBIBYTE, PrgType::FixedBank(Rom, BankIndex::LAST)),
]);

const CHR_LAYOUT: ChrLayout = ChrLayout::new(&[
    ChrWindow::new(0x0000, 0x1FFF, 8 * KIBIBYTE, ChrType::FixedBank(Rom, BankIndex::FIRST)),
]);

// Similar to UxROM.
pub struct Mapper071;

impl Mapper for Mapper071 {
    fn initial_layout(&self) -> InitialLayout {
        InitialLayout::builder()
            .prg_max_bank_count(16)
            .prg_bank_size(16 * KIBIBYTE)
            .prg_windows(PRG_LAYOUT)
            .chr_max_bank_count(1)
            .chr_bank_size(8 * KIBIBYTE)
            .chr_windows(CHR_LAYOUT)
            .name_table_mirroring_source(NameTableMirroringSource::Cartridge)
            .build()
    }

    fn write_to_cartridge_space(&mut self, params: &mut MapperParams, address: CpuAddress, value: u8) {
        let bank_index = value & 0b1111;
        match address.to_raw() {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x8FFF => { /* Do nothing. */ }
            // https://www.nesdev.org/wiki/INES_Mapper_071#Mirroring_($8000-$9FFF)
            0x9000..=0x9FFF => {
                let mirroring = if value & 0b0001_0000 == 0 {
                    NameTableMirroring::OneScreenLeftBank
                } else {
                    NameTableMirroring::OneScreenRightBank
                };
                params.set_name_table_mirroring(mirroring);
            }
            0xA000..=0xBFFF => { /* Do nothing. */ }
            0xC000..=0xFFFF => params.set_bank_index_register(P0, bank_index),
        }
    }
}
