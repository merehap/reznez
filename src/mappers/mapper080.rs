use crate::mapper::*;

pub const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(2048 * KIBIBYTE)
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7EFF, 7 * KIBIBYTE + 3 * KIBIBYTE / 4, PrgBank::EMPTY),
        PrgWindow::new(0x7F00, 0x7F7F, KIBIBYTE / 8, PrgBank::WORK_RAM.fixed_index(0).status_register(S0)),
        PrgWindow::new(0x7F80, 0x7FFF, KIBIBYTE / 8, PrgBank::WORK_RAM.fixed_index(0).status_register(S0)),
        PrgWindow::new(0x8000, 0x9FFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P0)),
        PrgWindow::new(0xA000, 0xBFFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P1)),
        PrgWindow::new(0xC000, 0xDFFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P2)),
        PrgWindow::new(0xE000, 0xFFFF, 8 * KIBIBYTE, PrgBank::ROM.fixed_index(-1)),
    ])
    .chr_rom_max_size(256 * KIBIBYTE)
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x07FF, 2 * KIBIBYTE, ChrBank::ROM.switchable(C0)),
        ChrWindow::new(0x0800, 0x0FFF, 2 * KIBIBYTE, ChrBank::ROM.switchable(C1)),
        ChrWindow::new(0x1000, 0x13FF, 1 * KIBIBYTE, ChrBank::ROM.switchable(C2)),
        ChrWindow::new(0x1400, 0x17FF, 1 * KIBIBYTE, ChrBank::ROM.switchable(C3)),
        ChrWindow::new(0x1800, 0x1BFF, 1 * KIBIBYTE, ChrBank::ROM.switchable(C4)),
        ChrWindow::new(0x1C00, 0x1FFF, 1 * KIBIBYTE, ChrBank::ROM.switchable(C5)),
    ])
    .name_table_mirrorings(&[
        NameTableMirroring::HORIZONTAL,
        NameTableMirroring::VERTICAL,
    ])
    .read_write_statuses(&[
        ReadWriteStatus::Disabled,
        ReadWriteStatus::ReadWrite,
    ])
    .build();

// Taito's X1-005
pub struct Mapper080;

impl Mapper for Mapper080 {
    fn write_register(&mut self, mem: &mut Memory, addr: CpuAddress, value: u8) {
        match *addr {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x7EEF => { /* Do nothing. */ }
            0x7EF0 => mem.set_chr_register(C0, value),
            0x7EF1 => mem.set_chr_register(C1, value),
            0x7EF2 => mem.set_chr_register(C2, value),
            0x7EF3 => mem.set_chr_register(C3, value),
            0x7EF4 => mem.set_chr_register(C4, value),
            0x7EF5 => mem.set_chr_register(C5, value),
            0x7EF6..=0x7EF7 => mem.set_name_table_mirroring(value & 1),
            0x7EF8..=0x7EF9 => {
                let ram_enabled = value == 0xA3;
                mem.set_read_write_status(S0, ram_enabled as u8);
            }
            0x7EFA..=0x7EFB => mem.set_prg_register(P0, value),
            0x7EFC..=0x7EFD => mem.set_prg_register(P1, value),
            0x7EFE..=0x7EFF => mem.set_prg_register(P2, value),
            0x7F00..=0xFFFF => { /* Do nothing. */ }
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}
