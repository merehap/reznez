use crate::mapper::*;

const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(128 * KIBIBYTE)
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::ABSENT),
        PrgWindow::new(0x8000, 0xBFFF, 16 * KIBIBYTE, PrgBank::ROM.switchable(P0)),
        PrgWindow::new(0xC000, 0xFFFF, 16 * KIBIBYTE, PrgBank::ROM.fixed_number(-1)),
    ])
    .chr_rom_max_size(128 * KIBIBYTE)
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x1FFF, 8 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C0)),
    ])
    .cartridge_selection_name_table_mirrorings([
        // All mapper 152 entries in nes20db.xml have horizontal mirroring.
        Some(NameTableMirroring::HORIZONTAL),
        // Unverified, no entries in nes20db.xml, no ROMs found.
        Some(NameTableMirroring::VERTICAL),
        // Unverified, no entries in nes20db.xml, no ROMs found.
        Some(NameTableMirroring::ONE_SCREEN_LEFT_BANK),
        // Unverified, no entries in nes20db.xml, no ROMs found.
        Some(NameTableMirroring::ONE_SCREEN_LEFT_BANK),
    ])
    .name_table_mirrorings(&[
        NameTableMirroring::ONE_SCREEN_LEFT_BANK,
        NameTableMirroring::ONE_SCREEN_RIGHT_BANK,
    ])
    .build();

// Similar to Mapper070, but with one screen mirroring control.
pub struct Mapper152;

impl Mapper for Mapper152 {
    fn has_bus_conflicts(&self) -> bool {
        true
    }

    fn write_register(&mut self, bus: &mut Bus, addr: CpuAddress, value: u8) {
        match *addr {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x7FFF => { /* Do nothing. */ },
            0x8000..=0xFFFF => {
                let fields = splitbits!(min=u8, value, "mpppcccc");
                bus.set_name_table_mirroring(fields.m);
                bus.set_prg_register(P0, fields.p);
                bus.set_chr_register(C0, fields.c);
            }
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}
