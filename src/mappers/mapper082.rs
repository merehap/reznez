use crate::mapper::*;

const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(128 * KIBIBYTE)
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x67FF, 2 * KIBIBYTE, PrgBank::WORK_RAM.fixed_index(0).status_register(S0)),
        PrgWindow::new(0x6800, 0x6FFF, 2 * KIBIBYTE, PrgBank::WORK_RAM.fixed_index(2).status_register(S1)),
        PrgWindow::new(0x7000, 0x73FF, 1 * KIBIBYTE, PrgBank::WORK_RAM.fixed_index(4).status_register(S2)),
        PrgWindow::new(0x7400, 0x7FFF, 3 * KIBIBYTE, PrgBank::ABSENT),
        PrgWindow::new(0x8000, 0x9FFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P0)),
        PrgWindow::new(0xA000, 0xBFFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P1)),
        PrgWindow::new(0xC000, 0xDFFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P2)),
        PrgWindow::new(0xE000, 0xFFFF, 8 * KIBIBYTE, PrgBank::ROM.fixed_index(-1)),
    ])
    .chr_rom_max_size(256 * KIBIBYTE)
    // Large windows first.
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x07FF, 2 * KIBIBYTE, ChrBank::ROM.switchable(C0)),
        ChrWindow::new(0x0800, 0x0FFF, 2 * KIBIBYTE, ChrBank::ROM.switchable(C1)),
        ChrWindow::new(0x1000, 0x13FF, 1 * KIBIBYTE, ChrBank::ROM.switchable(C2)),
        ChrWindow::new(0x1400, 0x17FF, 1 * KIBIBYTE, ChrBank::ROM.switchable(C3)),
        ChrWindow::new(0x1800, 0x1BFF, 1 * KIBIBYTE, ChrBank::ROM.switchable(C4)),
        ChrWindow::new(0x1C00, 0x1FFF, 1 * KIBIBYTE, ChrBank::ROM.switchable(C5)),
    ])
    // Small windows first.
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x03FF, 1 * KIBIBYTE, ChrBank::ROM.switchable(C2)),
        ChrWindow::new(0x0400, 0x07FF, 1 * KIBIBYTE, ChrBank::ROM.switchable(C3)),
        ChrWindow::new(0x0800, 0x0BFF, 1 * KIBIBYTE, ChrBank::ROM.switchable(C4)),
        ChrWindow::new(0x0C00, 0x0FFF, 1 * KIBIBYTE, ChrBank::ROM.switchable(C5)),
        ChrWindow::new(0x1000, 0x17FF, 2 * KIBIBYTE, ChrBank::ROM.switchable(C0)),
        ChrWindow::new(0x1800, 0x1FFF, 2 * KIBIBYTE, ChrBank::ROM.switchable(C1)),
    ])
    .name_table_mirrorings(&[
        NameTableMirroring::HORIZONTAL,
        NameTableMirroring::VERTICAL,
    ])
    .read_write_statuses(&[
        ReadWriteStatus::ReadOnlyZeros,
        ReadWriteStatus::ReadWrite,
    ])
    .build();

const READ_ONLY_ZEROS: u8 = 0;
const READ_WRITE: u8 = 1;

// Taito X1-017
// TODO: Read back 0 instead of open bus in all cases.
// TODO: Implement IRQ (even though it's not used in any commercial games).
pub struct Mapper082;

impl Mapper for Mapper082 {
    fn write_register(&mut self, mem: &mut Memory, addr: CpuAddress, value: u8) {
        match *addr {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x7EEF => { /* Do nothing. */ }
            0x7EF0 => mem.set_chr_register(C0, value & 0b1111_1110),
            0x7EF1 => mem.set_chr_register(C1, value & 0b1111_1110),
            0x7EF2 => mem.set_chr_register(C2, value),
            0x7EF3 => mem.set_chr_register(C3, value),
            0x7EF4 => mem.set_chr_register(C4, value),
            0x7EF5 => mem.set_chr_register(C5, value),
            0x7EF6 => {
                let fields = splitbits!(min=u8, value, "......lm");
                mem.set_chr_layout(fields.l);
                mem.set_name_table_mirroring(fields.m);
            }
            0x7EF7 => {
                let prg_read_write_status = if value == 0xCA { READ_WRITE } else { READ_ONLY_ZEROS };
                mem.set_read_write_status(S0, prg_read_write_status);
            }
            0x7EF8 => {
                let prg_read_write_status = if value == 0x69 { READ_WRITE } else { READ_ONLY_ZEROS };
                mem.set_read_write_status(S1, prg_read_write_status);
            }
            0x7EF9 => {
                let prg_read_write_status = if value == 0x84 { READ_WRITE } else { READ_ONLY_ZEROS };
                mem.set_read_write_status(S2, prg_read_write_status);
            }
            0x7EFA => mem.set_prg_register(P0, splitbits_named!(value, "..pppp..")),
            0x7EFB => mem.set_prg_register(P1, splitbits_named!(value, "..pppp..")),
            0x7EFC => mem.set_prg_register(P2, splitbits_named!(value, "..pppp..")),
            0x7EFD => { /* IRQ not yet implemented. */ }
            0x7EFE => { /* IRQ not yet implemented. */ }
            0x7EFF => { /* IRQ not yet implemented. */ }
            0x7F00..=0xFFFF => { /* Do nothing. */ }
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}
