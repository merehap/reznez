use crate::memory::mapper::*;

const LAYOUT: Layout = Layout::builder()
    .prg_max_bank_count(1)
    .chr_max_bank_count(2)
    .name_table_mirroring_source(NameTableMirroringSource::Cartridge)
    .prg_layouts(&[
        PrgLayout::new(&[
            PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, Bank::EMPTY),
            PrgWindow::new(0x8000, 0xFFFF, 32 * KIBIBYTE, Bank::fixed_rom(BankIndex::FIRST)),
        ])
    ])
    .chr_layouts(&[
        ChrLayout::new(&[
            ChrWindow::new(0x0000, 0x1FFF, 8 * KIBIBYTE, Bank::switchable_rom(C0)),
        ])
    ])
    .build();

// SA-0036 - Taiwan Mahjong 16
pub struct Mapper149;

impl Mapper for Mapper149 {
    fn write_to_cartridge_space(&mut self, params: &mut MapperParams, address: CpuAddress, value: u8) {
        match address.to_raw() {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x7FFF => { /* Do nothing. */ }
            0x8000..=0xFFFF => params.set_bank_register(C0, splitbits_named!(value, "c.......")),
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}
