use crate::mapper::*;

const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(512 * KIBIBYTE)
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::ABSENT),
        PrgWindow::new(0x8000, 0xFFFF, 32 * KIBIBYTE, PrgBank::ROM.switchable(P0)),
    ])
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::ABSENT),
        PrgWindow::new(0x8000, 0xBFFF, 16 * KIBIBYTE, PrgBank::ROM.switchable(P0)),
        PrgWindow::new(0xC000, 0xFFFF, 16 * KIBIBYTE, PrgBank::ROM.switchable(P0)),
    ])
    .chr_rom_max_size(8 * KIBIBYTE)
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x1FFF, 8 * KIBIBYTE, ChrBank::ROM_OR_RAM),
    ])
    .name_table_mirrorings(&[
        // L L
        // L R
        NameTableMirroring::new(
            NameTableSource::Ciram(CiramSide::Left),
            NameTableSource::Ciram(CiramSide::Left),
            NameTableSource::Ciram(CiramSide::Left),
            NameTableSource::Ciram(CiramSide::Right),
        ),
        NameTableMirroring::VERTICAL,
        NameTableMirroring::HORIZONTAL,
        NameTableMirroring::ONE_SCREEN_RIGHT_BANK,
    ])
    .build();

// Weird Super 42-in-1
// Untested. Confused documentation. Super 42-in-1 is too big for the mapper as documented.
pub struct Mapper233;

impl Mapper for Mapper233 {
    fn write_register(&mut self, bus: &mut Bus, addr: CpuAddress, value: u8) {
        match *addr {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x5FFF => { /* No regs here. */ }
            0x6000..=0xFFFF => {
                let (mirroring, layout, prg_bank) = splitbits_named!(min=u8, value, "mmlp pppp");
                bus.set_name_table_mirroring(mirroring);
                bus.set_prg_layout(layout);
                bus.set_prg_register(P0, prg_bank);
            }
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}