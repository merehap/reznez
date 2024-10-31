use crate::memory::mapper::*;

const LAYOUT: Layout = Layout::builder()
    .prg_max_size(256 * KIBIBYTE)
    .chr_max_size(128 * KIBIBYTE)
    .override_meta_register(M0, C1)
    .override_meta_register(M1, C3)
    .prg_layout(&[
        Window::new(0x6000, 0x7FFF,  8 * KIBIBYTE, Bank::WORK_RAM),
        Window::new(0x8000, 0xBFFF, 16 * KIBIBYTE, Bank::ROM.switchable(P0)),
        Window::new(0xC000, 0xFFFF, 16 * KIBIBYTE, Bank::ROM.fixed_index(-1)),
    ])
    .chr_layout(&[
        Window::new(0x0000, 0x0FFF, 4 * KIBIBYTE, Bank::ROM.meta_switchable(M0)),
        Window::new(0x1000, 0x1FFF, 4 * KIBIBYTE, Bank::ROM.meta_switchable(M1)),
    ])
    .build();

const MIRRORINGS: [NameTableMirroring; 2] = [
    NameTableMirroring::Vertical,
    NameTableMirroring::Horizontal,
];

// MMC4 (FxROM) - Similar to MMC2, but with Work RAM, bigger PRG ROM windows, and different bank-switching.
pub struct Mapper010;

impl Mapper for Mapper010 {
    fn write_to_cartridge_space(&mut self, params: &mut MapperParams, address: CpuAddress, value: u8) {
        let bank_index = value & 0b0001_1111;
        match address.to_raw() {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x5FFF => { /* Do nothing. */ }
            0x6000..=0x7FFF => params.write_prg(address, value),
            0x8000..=0x9FFF => { /* Do nothing. */ }
            0xA000..=0xAFFF => params.set_bank_register(P0, bank_index & 0b0000_1111),
            0xB000..=0xBFFF => params.set_bank_register(C0, bank_index),
            0xC000..=0xCFFF => params.set_bank_register(C1, bank_index),
            0xD000..=0xDFFF => params.set_bank_register(C2, bank_index),
            0xE000..=0xEFFF => params.set_bank_register(C3, bank_index),
            0xF000..=0xFFFF => params.set_name_table_mirroring(MIRRORINGS[usize::from(value & 1)]),
        }
    }

    fn on_ppu_read(&mut self, params: &mut MapperParams, address: PpuAddress, _value: u8) {
        let (meta_id, bank_register_id) = match address.to_u16() {
            0x0FD8..=0x0FDF => (M0, C0),
            0x0FE8..=0x0FEF => (M0, C1),
            0x1FD8..=0x1FDF => (M1, C2),
            0x1FE8..=0x1FEF => (M1, C3),
            // Skip to standard CHR memory operation.
            _ => return,
        };

        params.set_meta_register(meta_id, bank_register_id);
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}
