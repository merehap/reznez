use crate::memory::mapper::*;

const PRG_LAYOUT: PrgLayout = PrgLayout::new(&[
    PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, Bank::EMPTY),
    PrgWindow::new(0x8000, 0xFFFF, 32 * KIBIBYTE, Bank::switchable_rom(P0)),
]);

const CHR_LAYOUT: ChrLayout = ChrLayout::new(&[
    ChrWindow::new(0x0000, 0x1FFF, 8 * KIBIBYTE, Bank::fixed_ram(BankIndex::FIRST)),
]);

// BNROM (BxROM): Irem I-IM and NES-BNROM boards
pub struct Mapper034_2;

impl Mapper for Mapper034_2 {
    fn layout(&self) -> Layout {
        Layout::builder()
            // Oversize definition for BxROM. The actual BNROM cartridge only supports 2 banks.
            .prg_max_bank_count(256)
            .prg_bank_size(32 * KIBIBYTE)
            .prg_layout(PRG_LAYOUT)
            .chr_max_bank_count(1)
            .chr_bank_size(8 * KIBIBYTE)
            .chr_layout(CHR_LAYOUT)
            .name_table_mirroring_source(NameTableMirroringSource::Cartridge)
            // TODO: Verify if this is necessary. Might only be used for NINA-001.
            .override_bank_register(C1, BankIndex::LAST)
            .build()
    }

    fn has_bus_conflicts(&self) -> HasBusConflicts {
        HasBusConflicts::Yes
    }

    fn write_to_cartridge_space(&mut self, params: &mut MapperParams, address: CpuAddress, value: u8) {
        match address.to_raw() {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x7FFF => { /* Do nothing. */ }
            0x8000..=0xFFFF => params.set_bank_register(P0, value),
        }
    }
}
