use crate::memory::mapper::*;

const LAYOUT: Layout = Layout::builder()
    // Oversize definition. The actual cartridge only uses 2 banks.
    .prg_max_bank_count(256)
    .prg_layouts(&[
        PrgLayout::new(&[
            PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, Bank::EMPTY),
            PrgWindow::new(0x8000, 0xFFFF, 32 * KIBIBYTE, Bank::switchable_ram(P0)),
        ])
    ])
    // Oversize definition. The actual cartridge only uses 16 banks.
    .chr_max_bank_count(256)
    .chr_layouts(&[
        ChrLayout::new(&[
            ChrWindow::new(0x0000, 0x0FFF, 4 * KIBIBYTE, Bank::switchable_rom(C0)),
            ChrWindow::new(0x1000, 0x1FFF, 4 * KIBIBYTE, Bank::switchable_rom(C1)),
        ])
    ])
    .name_table_mirroring_source(NameTableMirroringSource::Cartridge)
    // TODO: Verify if this is necessary. Might only be used for BxROM.
    .override_bank_register(C1, BankIndex::LAST)
    .build();


// NINA-01
pub struct Mapper034_1;

impl Mapper for Mapper034_1 {
    fn write_to_cartridge_space(&mut self, params: &mut MapperParams, address: CpuAddress, value: u8) {
        match address.to_raw() {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x7FFC => { /* Do nothing. */ }
            0x7FFD => params.set_bank_register(P0, value),
            0x7FFE => params.set_bank_register(C0, value),
            0x7FFF => params.set_bank_register(C1, value),
            0x8000..=0xFFFF => { /* Do nothing. */ }
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}
