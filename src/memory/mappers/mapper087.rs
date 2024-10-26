use crate::memory::mapper::*;

const LAYOUT: Layout = Layout::builder()
    .prg_max_bank_count(1)
    .prg_bank_size(32 * KIBIBYTE)
    .prg_layouts(&[
        PrgLayout::new(&[
            PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, Bank::EMPTY),
            PrgWindow::new(0x8000, 0xFFFF, 32 * KIBIBYTE, Bank::fixed_rom(BankIndex::FIRST)),
        ])
    ])
    .chr_max_bank_count(256)
    .chr_bank_size(8 * KIBIBYTE)
    .chr_layouts(&[
        ChrLayout::new(&[
            ChrWindow::new(0x0000, 0x1FFF, 8 * KIBIBYTE, Bank::switchable_rom(C0)),
        ])
    ])
    .name_table_mirroring_source(NameTableMirroringSource::Cartridge)
    .build();

// Similar to CNROM.
pub struct Mapper087;

impl Mapper for Mapper087 {
    fn write_to_cartridge_space(&mut self, params: &mut MapperParams, cpu_address: CpuAddress, value: u8) {
        // Swap the low two bits, ignore the rest.
        let bank_index = ((value & 0b10) >> 1) | ((value & 0b01) << 1);
        match cpu_address.to_raw() {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x5FFF => { /* Do nothing. */ }
            0x6000..=0x7FFF => params.set_bank_register(C0, bank_index),
            0x8000..=0xFFFF => { /* Do nothing. */ }
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}
