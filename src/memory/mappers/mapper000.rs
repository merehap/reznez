use crate::memory::mapper::*;

const LAYOUT: Layout = Layout::builder()
    .prg_max_size(32 * KIBIBYTE)
    .chr_max_size(8 * KIBIBYTE)
    .name_table_mirroring_source(NameTableMirroringSource::Cartridge)
    .prg_layout(PrgLayout::new(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, Bank::EMPTY),
        PrgWindow::new(0x8000, 0xFFFF, 32 * KIBIBYTE, Bank::fixed_rom(BankIndex::FIRST)),
    ]))
    .chr_layout(ChrLayout::new(&[
        ChrWindow::new(0x0000, 0x1FFF, 8 * KIBIBYTE, Bank::fixed_rom(BankIndex::FIRST)),
    ]))
    .build();

// NROM
// The simplest mapper. Defined entirely by its Layout, it doesn't actually have any custom logic.
pub struct Mapper000;

impl Mapper for Mapper000 {
    fn write_to_cartridge_space(&mut self, _params: &mut MapperParams, address: CpuAddress, _value: u8) {
        match address.to_raw() {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0xFFFF => { /* Only mapper 0 does nothing here. */ }
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}
