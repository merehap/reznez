use crate::memory::mapper::*;

const PRG_LAYOUT: PrgLayout = PrgLayout::new(&[
    // TODO: PlayChoice uses this window.
    PrgWindow::new(0x6000, 0x7FFF, 8 * KIBIBYTE, Bank::EMPTY),
    PrgWindow::new(0x8000, 0x9FFF, 8 * KIBIBYTE, Bank::switchable_rom(P0)),
    PrgWindow::new(0xA000, 0xBFFF, 8 * KIBIBYTE, Bank::fixed_rom(BankIndex::THIRD_LAST)),
    PrgWindow::new(0xC000, 0xDFFF, 8 * KIBIBYTE, Bank::fixed_rom(BankIndex::SECOND_LAST)),
    PrgWindow::new(0xE000, 0xFFFF, 8 * KIBIBYTE, Bank::fixed_rom(BankIndex::LAST)),
]);

const CHR_LAYOUT: ChrLayout = ChrLayout::new(&[
    ChrWindow::new(0x0000, 0x0FFF, 4 * KIBIBYTE, Bank::meta_switchable_rom(M0)),
    ChrWindow::new(0x1000, 0x1FFF, 4 * KIBIBYTE, Bank::meta_switchable_rom(M1)),
]);

const MIRRORINGS: [NameTableMirroring; 2] = [
    NameTableMirroring::Vertical,
    NameTableMirroring::Horizontal,
];

// MMC2 (PNROM and PEEOROM boards)
pub struct Mapper009;

impl Mapper for Mapper009 {
    fn layout(&self) -> Layout {
        Layout::builder()
            .prg_max_bank_count(32)
            .prg_bank_size(8 * KIBIBYTE)
            .prg_layout(PRG_LAYOUT)
            .chr_max_bank_count(256)
            .chr_bank_size(4 * KIBIBYTE)
            .chr_layout(CHR_LAYOUT)
            .name_table_mirroring_source(NameTableMirroringSource::Cartridge)
            .override_meta_register(M0, C1)
            .override_second_meta_register(M1, C3)
            .build()
    }

    fn write_to_cartridge_space(&mut self, params: &mut MapperParams, address: CpuAddress, value: u8) {
        let bank_index = value & 0b0001_1111;
        match address.to_raw() {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x9FFF => { /* Do nothing. */ }
            0xA000..=0xAFFF => params.set_bank_register(P0, bank_index),
            0xB000..=0xBFFF => params.set_bank_register(C0, bank_index),
            0xC000..=0xCFFF => params.set_bank_register(C1, bank_index),
            0xD000..=0xDFFF => params.set_bank_register(C2, bank_index),
            0xE000..=0xEFFF => params.set_bank_register(C3, bank_index),
            0xF000..=0xFFFF => params.set_name_table_mirroring(MIRRORINGS[usize::from(value & 1)]),
        }
    }

    fn on_ppu_read(&mut self, params: &mut MapperParams, address: PpuAddress, _value: u8) {
        let (meta_id, bank_register_id) = match address.to_u16() {
            0x0FD8 => (M0, C0),
            0x0FE8 => (M0, C1),
            0x1FD8..=0x1FDF => (M1, C2),
            0x1FE8..=0x1FEF => (M1, C3),
            // Skip to standard CHR memory operation.
            _ => return,
        };

        params.set_meta_register(meta_id, bank_register_id);
    }
}
