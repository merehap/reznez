use crate::memory::mapper::*;

const PRG_WINDOWS: PrgWindows = PrgWindows::new(&[
    PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::Empty),
    PrgWindow::new(0x8000, 0xFFFF, 32 * KIBIBYTE, PrgBank::Fixed(Rom, BankIndex::FIRST)),
]);

const CHR_WINDOWS: ChrWindows = ChrWindows::new(&[
    ChrWindow::new(0x0000, 0x1FFF, 8 * KIBIBYTE, ChrBank::Fixed(Rom, BankIndex::FIRST)),
]);

// NROM
pub struct Mapper000;

impl Mapper for Mapper000 {
    fn initial_layout(&self) -> InitialLayout {
        InitialLayout::builder()
            .prg_max_bank_count(1)
            .prg_bank_size(32 * KIBIBYTE)
            .prg_windows(PRG_WINDOWS)
            .chr_max_bank_count(1)
            .chr_bank_size(8 * KIBIBYTE)
            .chr_windows(CHR_WINDOWS)
            .name_table_mirroring_source(NameTableMirroringSource::Cartridge)
            .build()
    }

    fn write_to_cartridge_space(&mut self, _params: &mut MapperParams, address: CpuAddress, _value: u8) {
        match address.to_raw() {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0xFFFF => { /* Only mapper 0 does nothing here. */ }
        }
    }
}
