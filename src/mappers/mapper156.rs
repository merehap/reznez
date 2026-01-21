use crate::mapper::*;

const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(256 * KIBIBYTE)
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::RAM_OR_ABSENT),
        PrgWindow::new(0x8000, 0xBFFF, 16 * KIBIBYTE, PrgBank::ROM.switchable(P0)),
        PrgWindow::new(0xC000, 0xFFFF, 16 * KIBIBYTE, PrgBank::ROM.fixed_number(-1)),
    ])
    .chr_rom_max_size(512 * KIBIBYTE)
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x03FF, 1 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C0)),
        ChrWindow::new(0x0400, 0x07FF, 1 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C1)),
        ChrWindow::new(0x0800, 0x0BFF, 1 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C2)),
        ChrWindow::new(0x0C00, 0x0FFF, 1 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C3)),
        ChrWindow::new(0x1000, 0x13FF, 1 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C4)),
        ChrWindow::new(0x1400, 0x17FF, 1 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C5)),
        ChrWindow::new(0x1800, 0x1BFF, 1 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C6)),
        ChrWindow::new(0x1C00, 0x1FFF, 1 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C7)),
    ])
    .name_table_mirrorings(&[
        NameTableMirroring::VERTICAL,
        NameTableMirroring::HORIZONTAL,
        NameTableMirroring::ONE_SCREEN_LEFT_BANK,
        NameTableMirroring::ONE_SCREEN_LEFT_BANK,
    ])
    // "Mirroring defaults to one-screen."
    .cartridge_selection_name_table_mirrorings([
        Some(NameTableMirroring::ONE_SCREEN_LEFT_BANK),
        Some(NameTableMirroring::ONE_SCREEN_LEFT_BANK),
        Some(NameTableMirroring::ONE_SCREEN_LEFT_BANK),
        Some(NameTableMirroring::ONE_SCREEN_LEFT_BANK),
    ])
    .build();

// DAOU ROM Controller DIS23C01 DAOU 245
pub struct Mapper156;

impl Mapper for Mapper156 {
    fn write_register(&mut self, bus: &mut Bus, addr: CpuAddress, value: u8) {
        match *addr {
            0x0000..=0x401F => unreachable!(),
            0xC000 => bus.set_chr_register_low_byte(C0, value),
            0xC001 => bus.set_chr_register_low_byte(C1, value),
            0xC002 => bus.set_chr_register_low_byte(C2, value),
            0xC003 => bus.set_chr_register_low_byte(C3, value),
            0xC004 => bus.set_chr_register_high_byte(C0, value & 1),
            0xC005 => bus.set_chr_register_high_byte(C1, value & 1),
            0xC006 => bus.set_chr_register_high_byte(C2, value & 1),
            0xC007 => bus.set_chr_register_high_byte(C3, value & 1),
            0xC008 => bus.set_chr_register_low_byte(C4, value),
            0xC009 => bus.set_chr_register_low_byte(C5, value),
            0xC00A => bus.set_chr_register_low_byte(C6, value),
            0xC00B => bus.set_chr_register_low_byte(C7, value),
            0xC00C => bus.set_chr_register_high_byte(C4, value & 1),
            0xC00D => bus.set_chr_register_high_byte(C5, value & 1),
            0xC00E => bus.set_chr_register_high_byte(C6, value & 1),
            0xC00F => bus.set_chr_register_high_byte(C7, value & 1),
            0xC010 => bus.set_prg_register(P0, value & 0b1111),
            0xC014 => bus.set_name_table_mirroring(value & 0b11),
            _ => { /* Do nothing. */}
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}