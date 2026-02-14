use crate::mapper::*;

const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(256 * KIBIBYTE)
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::RAM_OR_ABSENT),
        PrgWindow::new(0x8000, 0xBFFF, 16 * KIBIBYTE, PrgBank::ROM.switchable(P)),
        PrgWindow::new(0xC000, 0xFFFF, 16 * KIBIBYTE, PrgBank::ROM.fixed_number(-1)),
    ])
    .chr_rom_max_size(512 * KIBIBYTE)
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x03FF, 1 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C)),
        ChrWindow::new(0x0400, 0x07FF, 1 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(D)),
        ChrWindow::new(0x0800, 0x0BFF, 1 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(E)),
        ChrWindow::new(0x0C00, 0x0FFF, 1 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(F)),
        ChrWindow::new(0x1000, 0x13FF, 1 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(G)),
        ChrWindow::new(0x1400, 0x17FF, 1 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(H)),
        ChrWindow::new(0x1800, 0x1BFF, 1 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(I)),
        ChrWindow::new(0x1C00, 0x1FFF, 1 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(J)),
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
            0xC000 => bus.set_chr_register_low_byte(C, value),
            0xC001 => bus.set_chr_register_low_byte(D, value),
            0xC002 => bus.set_chr_register_low_byte(E, value),
            0xC003 => bus.set_chr_register_low_byte(F, value),
            0xC004 => bus.set_chr_register_high_byte(C, value & 1),
            0xC005 => bus.set_chr_register_high_byte(D, value & 1),
            0xC006 => bus.set_chr_register_high_byte(E, value & 1),
            0xC007 => bus.set_chr_register_high_byte(F, value & 1),
            0xC008 => bus.set_chr_register_low_byte(G, value),
            0xC009 => bus.set_chr_register_low_byte(H, value),
            0xC00A => bus.set_chr_register_low_byte(I, value),
            0xC00B => bus.set_chr_register_low_byte(J, value),
            0xC00C => bus.set_chr_register_high_byte(G, value & 1),
            0xC00D => bus.set_chr_register_high_byte(H, value & 1),
            0xC00E => bus.set_chr_register_high_byte(I, value & 1),
            0xC00F => bus.set_chr_register_high_byte(J, value & 1),
            0xC010 => bus.set_prg_register(P, value & 0b1111),
            0xC014 => bus.set_name_table_mirroring(value & 0b11),
            _ => { /* Do nothing. */}
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}