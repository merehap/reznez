use crate::memory::mapper::*;

const LAYOUT: Layout = Layout::builder()
    .prg_max_bank_count(8)
    .prg_bank_size(32 * KIBIBYTE)
    .prg_layouts(&[
        PrgLayout::new(&[
            PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, Bank::EMPTY),
            PrgWindow::new(0x8000, 0xFFFF, 32 * KIBIBYTE, Bank::switchable_rom(P0)),
        ])
    ])
    .chr_max_bank_count(16)
    .chr_bank_size(8 * KIBIBYTE)
    .chr_layouts(&[
        ChrLayout::new(&[
            ChrWindow::new(0x0000, 0x1FFF, 8 * KIBIBYTE, Bank::switchable_rom(C0)),
        ])
    ])
    .name_table_mirroring_source(NameTableMirroringSource::Cartridge)
    .build();

// Jaleco's JF-13
pub struct Mapper086;

impl Mapper for Mapper086 {
    fn write_to_cartridge_space(&mut self, params: &mut MapperParams, address: CpuAddress, value: u8) {
        let address = address.to_raw();
        match address {
            0x0000..=0x401F => unreachable!(),
            0x6000..=0x6FFF => {
                let banks = splitbits!(value, ".cpp..cc");
                params.set_bank_register(C0, banks.c);
                params.set_bank_register(P0, banks.p);
            }
            0x7000..=0x7FFF => { /* TODO: Audio control. */ }
            _ => { /* Do nothing. */ }
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}
