use crate::memory::mapper::*;

const PRG_WINDOWS: PrgLayout = PrgLayout::new(&[
    PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, Bank::EMPTY),
    PrgWindow::new(0x8000, 0xBFFF, 16 * KIBIBYTE, Bank::fixed_rom(BankIndex::FIRST)),
    PrgWindow::new(0xC000, 0xFFFF, 16 * KIBIBYTE, Bank::switchable_rom(P0)),
]);

const CHR_WINDOWS: ChrLayout = ChrLayout::new(&[
    ChrWindow::new(0x0000, 0x1FFF, 8 * KIBIBYTE, Bank::fixed_rom(BankIndex::FIRST)),
]);

// UNROM, but the fixed bank and the switchable bank are swapped.
pub struct Mapper180;

impl Mapper for Mapper180 {
    fn initial_layout(&self) -> InitialLayout {
        InitialLayout::builder()
            .prg_max_bank_count(256)
            .prg_bank_size(16 * KIBIBYTE)
            .prg_windows(PRG_WINDOWS)
            .chr_max_bank_count(1)
            .chr_bank_size(8 * KIBIBYTE)
            .chr_windows(CHR_WINDOWS)
            .name_table_mirroring_source(NameTableMirroringSource::Cartridge)
            .build()
    }

    fn write_to_cartridge_space(&mut self, params: &mut MapperParams, address: CpuAddress, value: u8) {
        match address.to_raw() {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x7FFF => { /* Do nothing. */ },
            0x8000..=0xFFFF => params.set_bank_register(P0, value & 0b0000_0111),
        }
    }
}
