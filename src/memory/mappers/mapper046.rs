use crate::memory::mapper::*;

const PRG_LAYOUT: PrgLayout = PrgLayout::new(&[
    PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, Bank::EMPTY),
    PrgWindow::new(0x8000, 0xFFFF, 32 * KIBIBYTE, Bank::switchable_rom(P0)),
]);

const CHR_LAYOUT: ChrLayout = ChrLayout::new(&[
    ChrWindow::new(0x0000, 0x1FFF, 8 * KIBIBYTE, Bank::switchable_rom(C0)),
]);

// Rumble Station (Color Dreams).
// NOTE: Untested.
pub struct Mapper046 {
    prg_high_bits: u8,
    chr_high_bits: u8,
}

impl Mapper for Mapper046 {
    fn initial_layout(&self) -> InitialLayout {
        InitialLayout::builder()
            .prg_max_bank_count(32)
            .prg_bank_size(32 * KIBIBYTE)
            .prg_windows(PRG_LAYOUT)
            .chr_max_bank_count(128)
            .chr_bank_size(8 * KIBIBYTE)
            .chr_windows(CHR_LAYOUT)
            .name_table_mirroring_source(NameTableMirroringSource::Cartridge)
            .build()
    }

    fn write_to_cartridge_space(&mut self, params: &mut MapperParams, cpu_address: CpuAddress, value: u8) {
        match cpu_address.to_raw() {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x5FFF => { /* Do nothing. */ }
            0x6000..=0x7FFF => {
                self.prg_high_bits = (value & 0b1111_0000) >> 3;
                self.chr_high_bits = (value & 0b0000_1111) << 3;
            }
            0x8000..=0xFFFF => {
                let prg_bank_index = self.prg_high_bits | (value & 0b0000_0001);
                params.set_bank_register(P0, prg_bank_index);
                let chr_bank_index = self.chr_high_bits | ((value << 1) >> 5);
                params.set_bank_register(C0, chr_bank_index);
            }
        }
    }
}

impl Mapper046 {
    pub fn new() -> Self {
        Self {
            prg_high_bits: 0b0000_0000,
            chr_high_bits: 0b0000_0000,
        }
    }
}
