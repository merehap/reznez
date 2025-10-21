use crate::mapper::*;

const LAYOUT: Layout = Layout::builder()
    .override_prg_bank_register(P1, 0b10)
    .override_prg_bank_register(P2, 0b1110)
    // NROM-256
    .prg_rom_max_size(1024 * KIBIBYTE)
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::ABSENT),
        PrgWindow::new(0x8000, 0xBFFF, 16 * KIBIBYTE, PrgBank::ROM.switchable(P0)),
        // P1 = P0 | 0b10
        PrgWindow::new(0xC000, 0xFFFF, 16 * KIBIBYTE, PrgBank::ROM.switchable(P1)),
    ])
    // UNROM
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::ABSENT),
        PrgWindow::new(0x8000, 0xBFFF, 16 * KIBIBYTE, PrgBank::ROM.switchable(P0)),
        // P2 = P0 | 0b1110
        PrgWindow::new(0xC000, 0xFFFF, 16 * KIBIBYTE, PrgBank::ROM.switchable(P2)),
    ])
    // NROM-64
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF, 8 * KIBIBYTE, PrgBank::ABSENT),
        // Mirrored, for a total of 4 instances.
        PrgWindow::new(0x8000, 0x9FFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P0)),
        PrgWindow::new(0xA000, 0xBFFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P0)),
        PrgWindow::new(0xC000, 0xDFFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P0)),
        PrgWindow::new(0xE000, 0xFFFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P0)),
    ])
    // NROM-128
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::ABSENT),
        // Mirrored, for a total of 2 instances.
        PrgWindow::new(0x8000, 0xBFFF, 16 * KIBIBYTE, PrgBank::ROM.switchable(P0)),
        PrgWindow::new(0xC000, 0xFFFF, 16 * KIBIBYTE, PrgBank::ROM.switchable(P0)),
    ])
    .chr_rom_max_size(8 * KIBIBYTE)
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x1FFF, 8 * KIBIBYTE, ChrBank::ROM_OR_RAM.fixed_index(0).status_register(S0)),
    ])
    .name_table_mirrorings(&[
        NameTableMirroring::VERTICAL,
        NameTableMirroring::HORIZONTAL,
    ])
    .read_write_statuses(&[
        ReadWriteStatus::ReadOnly,
        ReadWriteStatus::ReadWrite,
    ])
    .build();

// K-1029 and K-1030P (multicart)
// See https://www.nesdev.org/w/index.php?title=INES_Mapper_015&oldid=3854 for documentation, the
// latest version of that page is incomprehensible.
pub struct Mapper015;

impl Mapper for Mapper015 {
    fn write_register(&mut self, mem: &mut Memory, addr: CpuAddress, value: u8) {
        match *addr {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x7FFF => { /* Do nothing. */ }
            0x8000..=0xFFFF => {
                let prg_layout_index = (*addr & 0b11) as u8;
                mem.set_prg_layout(prg_layout_index);
                let chr_ram_writable = matches!(prg_layout_index, 1 | 2);
                mem.set_read_write_status(S0, chr_ram_writable as u8);

                let (s, mirroring, p) = splitbits_named!(min=u8, value, "smpppppp");
                let prg_bank = if prg_layout_index == 2 {
                    // NROM-64
                    combinebits!("0pppppps")
                } else {
                    combinebits!("0pppppp0")
                };

                mem.set_name_table_mirroring(mirroring);
                mem.set_prg_register(P0, prg_bank);
                mem.set_prg_register(P1, prg_bank | 0b10);
                mem.set_prg_register(P2, prg_bank | 0b1110);
            }
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}
