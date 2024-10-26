use crate::memory::mapper::*;

const LAYOUT: Layout = Layout::builder()
    .prg_max_bank_count(16)
    .chr_max_bank_count(16)
    .name_table_mirroring_source(NameTableMirroringSource::Cartridge)
    .prg_layouts(&[
        PrgLayout::new(&[
            PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, Bank::EMPTY),
            PrgWindow::new(0x8000, 0xFFFF, 32 * KIBIBYTE, Bank::switchable_rom(P0)),
        ])
    ])
    .chr_layouts(&[
        ChrLayout::new(&[
            ChrWindow::new(0x0000, 0x1FFF, 8 * KIBIBYTE, Bank::switchable_rom(C0)),
        ])
    ])
    .build();

pub struct Mapper240;

impl Mapper for Mapper240 {
    fn write_to_cartridge_space(&mut self, params: &mut MapperParams, address: CpuAddress, value: u8) {
        let address = address.to_raw();
        match address {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x5FFF => {
                let banks = splitbits!(value, "ppppcccc");
                params.set_bank_register(P0, banks.p);
                params.set_bank_register(C0, banks.c);
            }
            0x6000..=0xFFFF => { /* Do nothing. */ }
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}
