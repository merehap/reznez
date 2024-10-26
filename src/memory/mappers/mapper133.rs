use crate::memory::mapper::*;

const LAYOUT: Layout = Layout::builder()
    .prg_max_bank_count(8)
    .prg_layouts(&[
        PrgLayout::new(&[
            PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, Bank::EMPTY),
            PrgWindow::new(0x8000, 0xFFFF, 32 * KIBIBYTE, Bank::switchable_rom(P0)),
        ])
    ])
    .chr_max_bank_count(16)
    .chr_layouts(&[
        ChrLayout::new(&[
            ChrWindow::new(0x0000, 0x1FFF, 8 * KIBIBYTE, Bank::switchable_rom(C0)),
        ])
    ])
    .name_table_mirroring_source(NameTableMirroringSource::Cartridge)
    .build();

// Sachen 3009
pub struct Mapper133;

impl Mapper for Mapper133 {
    fn write_to_cartridge_space(&mut self, params: &mut MapperParams, address: CpuAddress, value: u8) {
        match address.to_raw() & 0xE100 {
            0x0000..=0x401F => unreachable!(),
            0x4100 => {
                let banks = splitbits!(value, ".....pcc");
                params.set_bank_register(P0, banks.p as u8);
                params.set_bank_register(C0, banks.c);
            }
            _ => { /* Do nothing. */ }
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}
