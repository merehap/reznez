use crate::memory::mapper::*;

const PRG_WINDOWS: PrgWindows = PrgWindows::new(&[
    PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::WorkRam), //PrgBank::Fixed(Ram, BankIndex::FIRST)),
    PrgWindow::new(0x8000, 0xBFFF, 16 * KIBIBYTE, PrgBank::Switchable(Rom, P0)),
    PrgWindow::new(0xC000, 0xFFFF, 16 * KIBIBYTE, PrgBank::Fixed(Rom, BankIndex::LAST)),
]);

const CHR_WINDOWS: ChrWindows = ChrWindows::new(&[
    ChrWindow::new(0x0000, 0x0FFF, 4 * KIBIBYTE, ChrBank::MetaSwitchable(Rom, M0)),
    ChrWindow::new(0x1000, 0x1FFF, 4 * KIBIBYTE, ChrBank::MetaSwitchable(Rom, M1)),
]);

// MMC4 - Similar to MMC2, but with Work RAM, bigger PRG ROM windows, and different bank-switching.
pub struct Mapper010;

impl Mapper for Mapper010 {
    fn initial_layout(&self) -> InitialLayout {
        InitialLayout::builder()
            .prg_max_bank_count(16)
            .prg_bank_size(16 * KIBIBYTE)
            .prg_windows(PRG_WINDOWS)
            .chr_max_bank_count(256)
            .chr_bank_size(4 * KIBIBYTE)
            .chr_windows(CHR_WINDOWS)
            .name_table_mirroring_source(NameTableMirroringSource::Cartridge)
            .build()
    }

    fn write_to_cartridge_space(&mut self, params: &mut MapperParams, address: CpuAddress, value: u8) {
        let bank_index = value & 0b0001_1111;
        match address.to_raw() {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x9FFF => { /* Do nothing. */ }
            0xA000..=0xAFFF => params.prg_memory_mut().set_bank_index_register(P0, bank_index & 0b0000_1111),
            0xB000..=0xBFFF => params.chr_memory_mut().set_bank_index_register(C0, bank_index),
            0xC000..=0xCFFF => params.chr_memory_mut().set_bank_index_register(C1, bank_index),
            0xD000..=0xDFFF => params.chr_memory_mut().set_bank_index_register(C2, bank_index),
            0xE000..=0xEFFF => params.chr_memory_mut().set_bank_index_register(C3, bank_index),
            0xF000..=0xFFFF => {
                let mirroring = if value & 1 == 0 {
                    NameTableMirroring::Vertical
                } else {
                    NameTableMirroring::Horizontal
                };
                params.set_name_table_mirroring(mirroring);
            }
        }
    }

    fn on_ppu_read(&mut self, params: &mut MapperParams, address: PpuAddress, _value: u8) {
        let (meta_id, bank_index_register_id) = match address.to_u16() {
            0x0FD8..=0x0FDF => (M0, C0),
            0x0FE8..=0x0FEF => (M0, C1),
            0x1FD8..=0x1FDF => (M1, C2),
            0x1FE8..=0x1FEF => (M1, C3),
            // Skip to standard CHR memory operation.
            _ => return,
        };

        params.chr_memory_mut().set_meta_register(meta_id, bank_index_register_id);
    }
}
