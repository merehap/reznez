use crate::mapper::*;

const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(512 * KIBIBYTE)
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::EMPTY),
        PrgWindow::new(0x8000, 0xFFFF, 32 * KIBIBYTE, PrgBank::ROM.switchable(P0)),
    ])
    .chr_rom_max_size(32 * KIBIBYTE)
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x07FF, 2 * KIBIBYTE, ChrBank::ROM.switchable(C0)),
        ChrWindow::new(0x0800, 0x1FFF, 6 * KIBIBYTE, ChrBank::SaveRam(0x0800)),
    ])
    // Cartridges for some reason don't specify a CHR Save RAM size.
    .chr_save_ram_size(8 * KIBIBYTE)
    .initial_name_table_mirroring(NameTableMirroring::new(
        NameTableSource::SaveRam(0x0000),
        NameTableSource::SaveRam(0x0400),
        NameTableSource::Ciram(CiramSide::Left),
        NameTableSource::Ciram(CiramSide::Right),
    ))
    .build();

// Irem (Napoleon Senki)
pub struct Mapper077;

impl Mapper for Mapper077 {
    fn write_register(&mut self, params: &mut MapperParams, cpu_address: u16, value: u8) {
        match cpu_address {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x7FFF => { /* Do nothing. */ }
            0x8000..=0xFFFF => {
                let banks = splitbits!(value, "ccccpppp");
                params.set_chr_register(C0, banks.c);
                params.set_prg_register(P0, banks.p);
            }
        }
    }

    fn has_bus_conflicts(&self) -> HasBusConflicts {
        HasBusConflicts::Yes
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}
