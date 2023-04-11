use crate::memory::mapper::*;

const PRG_LAYOUT: PrgLayout = PrgLayout::new(&[
    PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgType::WorkRam),
    PrgWindow::new(0x8000, 0xFFFF, 32 * KIBIBYTE, PrgType::SwitchableBank(Rom, P0)),
]);

const CHR_LAYOUT: ChrLayout = ChrLayout::new(&[
    ChrWindow::new(0x0000, 0x1FFF, 8 * KIBIBYTE, ChrType::FixedBank(Ram, BankIndex::FIRST)),
]);

// BxROM with WorkRam
pub struct Mapper241;

impl Mapper for Mapper241 {
    fn initial_layout(&self) -> InitialLayout {
        InitialLayout::builder()
            .prg_max_bank_count(256)
            .prg_bank_size(32 * KIBIBYTE)
            .prg_windows(PRG_LAYOUT)
            .chr_max_bank_count(1)
            .chr_bank_size(8 * KIBIBYTE)
            .chr_windows(CHR_LAYOUT)
            .name_table_mirroring_source(NameTableMirroringSource::Cartridge)
            .build()
    }

    fn write_to_cartridge_space(&mut self, params: &mut MapperParams, address: CpuAddress, value: u8) {
        match address.to_raw() {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x7FFF => { /* Do nothing. */ }
            0x8000..=0xFFFF => params.prg_memory_mut().set_bank_index_register(P0, value),
        }
    }
}
