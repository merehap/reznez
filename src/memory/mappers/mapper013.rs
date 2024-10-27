use crate::memory::mapper::*;

const LAYOUT: Layout = Layout::builder()
    .prg_max_size(32 * KIBIBYTE)
    .chr_max_size(16 * KIBIBYTE)
    .name_table_mirroring_source(NameTableMirroringSource::Cartridge)
    .prg_layout(PrgLayout::new(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, Bank::EMPTY),
        PrgWindow::new(0x8000, 0xFFFF, 32 * KIBIBYTE, Bank::fixed_rom(BankIndex::FIRST)),
    ]))
    .chr_layout(ChrLayout::new(&[
        ChrWindow::new(0x0000, 0x0FFF, 4 * KIBIBYTE, Bank::fixed_ram(BankIndex::FIRST)),
        ChrWindow::new(0x1000, 0x1FFF, 4 * KIBIBYTE, Bank::switchable_ram(C0)),
    ]))
    .build();

// CPROM
pub struct Mapper013;

impl Mapper for Mapper013 {
    fn write_to_cartridge_space(&mut self, params: &mut MapperParams, cpu_address: CpuAddress, value: u8) {
        match cpu_address.to_raw() {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x7FFF => { /* Do nothing. */ },
            0x8000..=0xFFFF => params.set_bank_register(C0, value & 0b11),
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}
