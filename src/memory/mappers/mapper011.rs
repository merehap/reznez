use crate::memory::mapper::*;

const LAYOUT: Layout = Layout::builder()
    .prg_max_bank_count(4)
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

// Color Dreams. Same as GxROM except with different register locations.
pub struct Mapper011;

impl Mapper for Mapper011 {
    fn write_to_cartridge_space(&mut self, params: &mut MapperParams, cpu_address: CpuAddress, value: u8) {
        match cpu_address.to_raw() {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x7FFF => { /* Do nothing. */ },
            0x8000..=0xFFFF => {
                let ids = splitbits!(value, "cccc..pp");
                params.set_bank_register(C0, ids.c);
                params.set_bank_register(P0, ids.p);
            }
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}
