use crate::memory::mapper::*;

const PRG_LAYOUT: PrgLayout = PrgLayout::new(&[
    PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, Bank::EMPTY),
    PrgWindow::new(0x8000, 0xFFFF, 32 * KIBIBYTE, Bank::switchable_ram(P0)),
]);

const CHR_LAYOUT: ChrLayout = ChrLayout::new(&[
    ChrWindow::new(0x0000, 0x0FFF, 4 * KIBIBYTE, Bank::switchable_rom(C0)),
    ChrWindow::new(0x1000, 0x1FFF, 4 * KIBIBYTE, Bank::switchable_rom(C1)),
]);

// NINA-01
pub struct Mapper034_1;

impl Mapper for Mapper034_1 {
    fn initial_layout(&self) -> InitialLayout {
        InitialLayout::builder()
            // Oversize definition. The actual cartridge only uses 2 banks.
            .prg_max_bank_count(256)
            .prg_bank_size(32 * KIBIBYTE)
            .prg_windows(PRG_LAYOUT)
            // Oversize definition. The actual cartridge only uses 16 banks.
            .chr_max_bank_count(256)
            .chr_bank_size(4 * KIBIBYTE)
            .chr_windows(CHR_LAYOUT)
            .name_table_mirroring_source(NameTableMirroringSource::Cartridge)
            // TODO: Verify if this is necessary. Might only be used for BxROM.
            .override_bank_register(C1, BankIndex::LAST)
            .build()
    }

    fn write_to_cartridge_space(&mut self, params: &mut MapperParams, address: CpuAddress, value: u8) {
        match address.to_raw() {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x7FFC => { /* Do nothing. */ }
            0x7FFD => params.set_bank_register(P0, value),
            0x7FFE => params.set_bank_register(C0, value),
            0x7FFF => params.set_bank_register(C1, value),
            0x8000..=0xFFFF => { /* Do nothing. */ }
        }
    }
}
