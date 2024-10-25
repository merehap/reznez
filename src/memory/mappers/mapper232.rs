use crate::memory::mapper::*;

const PRG_LAYOUT: PrgLayout = PrgLayout::new(&[
    PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, Bank::EMPTY),
    PrgWindow::new(0x8000, 0xBFFF, 16 * KIBIBYTE, Bank::switchable_rom(P0)),
    PrgWindow::new(0xC000, 0xFFFF, 16 * KIBIBYTE, Bank::switchable_rom(P1)),
]);

const CHR_LAYOUT: ChrLayout = ChrLayout::new(&[
    ChrWindow::new(0x0000, 0x1FFF, 8 * KIBIBYTE, Bank::fixed_rom(BankIndex::FIRST)),
]);

// Similar to mapper 71.
pub struct Mapper232;

impl Mapper for Mapper232 {
    fn initial_layout(&self) -> InitialLayout {
        InitialLayout::builder()
            .prg_max_bank_count(16)
            .prg_bank_size(16 * KIBIBYTE)
            .prg_layout(PRG_LAYOUT)
            .chr_max_bank_count(1)
            .chr_bank_size(8 * KIBIBYTE)
            .chr_layout(CHR_LAYOUT)
            .name_table_mirroring_source(NameTableMirroringSource::Cartridge)
            // The last bank for any of the mapper 232 PRG "blocks".
            .override_bank_register(P1, BankIndex::from_u8(0b11))
            .build()
    }

    fn write_to_cartridge_space(&mut self, params: &mut MapperParams, address: CpuAddress, value: u8) {
        let value = u16::from(value);
        match address.to_raw() {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x7FFF => { /* Do nothing. */ }
            0x8000..=0xBFFF => {
                let set_high_bank_bits = |bank_index| {
                    (bank_index & 0b0011) | ((value & 0b1_1000) >> 1)
                };
                params.update_bank_register(P0, &set_high_bank_bits);
                params.update_bank_register(P1, &set_high_bank_bits);
            }
            0xC000..=0xFFFF => {
                let set_low_bank_bits = |bank_index| {
                    (bank_index & 0b1100) | (value & 0b0011)
                };
                params.update_bank_register(P0, &set_low_bank_bits);
            }
        }
    }
}
