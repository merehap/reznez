use crate::memory::mapper::*;
use crate::memory::mappers::mapper206;
use crate::memory::mappers::mapper206::Mapper206;
//
// DxROM, Tengen MIMIC-1, Namco 118
// A much simpler predecessor to MMC3.
pub struct Mapper088 {
    mapper206: Mapper206,
    extended_chr_present: bool,
}

impl Mapper for Mapper088 {
    fn initial_layout(&self) -> InitialLayout {
        InitialLayout::builder()
            .prg_max_bank_count(16)
            .prg_bank_size(8 * KIBIBYTE)
            .prg_windows(mapper206::PRG_LAYOUT)
            // Doubled CHR, the only part of the layout different from Mapper206.
            .chr_max_bank_count(128)
            .chr_bank_size(1 * KIBIBYTE)
            .chr_windows(mapper206::CHR_LAYOUT)
            .name_table_mirroring_source(NameTableMirroringSource::Cartridge)
            .build()
    }

    fn write_to_cartridge_space(&mut self, params: &mut MapperParams, address: CpuAddress, value: u8) {
        self.mapper206.write_to_cartridge_space(params, address, value);

        let bank_index_updated = matches!(address.to_raw(), 0x8000..=0x9FFF) && address.is_odd();
        let extended_chr_register_selected =
            matches!(self.mapper206.selected_register_id(), C2 | C3 | C4 | C5);

        // If Mapper206 just wrote a new BankIndex, then for registers C2-C5 in Mapper088, the new
        // BankIndex must be modified to read from the second half of CHR.
        if self.extended_chr_present && bank_index_updated && extended_chr_register_selected {
            params.update_bank_index_register(
                self.mapper206.selected_register_id(),
                &|bank_index| bank_index | 0b0100_0000,
            );
        }
    }
}

impl Mapper088 {
    pub fn new(cartridge: &Cartridge) -> Self {
        Self {
            mapper206: Mapper206::new(),
            extended_chr_present: cartridge.chr_rom().len() > 64 * KIBIBYTE,
        }
    }
}
