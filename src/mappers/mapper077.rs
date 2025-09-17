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
    // TODO: The above is probably not true (it was probably caused by a bug). Verify that this line isn't needed.
    // TODO: I'm not seeing where this mapper is specified to use CHR Save RAM at all. Is it CHR Work RAM instead? That's what Mesen does.
    // TODO: Verify that NameTableSource::WorkRam actually works (SaveRam is panicking).
    .chr_save_ram_size(8 * KIBIBYTE)
    .four_screen_mirroring_definition(NameTableMirroring::new(
        NameTableSource::SaveRam(0x0000),
        NameTableSource::SaveRam(0x0400),
        NameTableSource::Ciram(CiramSide::Left),
        NameTableSource::Ciram(CiramSide::Right),
    ))
    .build();

// Irem (Napoleon Senki)
pub struct Mapper077;

impl Mapper for Mapper077 {
    fn write_register(&mut self, mem: &mut Memory, addr: CpuAddress, value: u8) {
        match *addr {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x7FFF => { /* Do nothing. */ }
            0x8000..=0xFFFF => {
                let banks = splitbits!(value, "ccccpppp");
                mem.set_chr_register(C0, banks.c);
                mem.set_prg_register(P0, banks.p);
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
