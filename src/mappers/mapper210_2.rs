use crate::mapper::*;

const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(512 * KIBIBYTE)
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF, 8 * KIBIBYTE, PrgBank::EMPTY),
        PrgWindow::new(0x8000, 0x9FFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P0)),
        PrgWindow::new(0xA000, 0xBFFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P1)),
        PrgWindow::new(0xC000, 0xDFFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P2)),
        PrgWindow::new(0xE000, 0xFFFF, 8 * KIBIBYTE, PrgBank::ROM.fixed_index(-1)),
    ])
    .chr_rom_max_size(256 * KIBIBYTE)
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x03FF, 1 * KIBIBYTE, ChrBank::ROM.switchable(C0)),
        ChrWindow::new(0x0400, 0x07FF, 1 * KIBIBYTE, ChrBank::ROM.switchable(C1)),
        ChrWindow::new(0x0800, 0x0BFF, 1 * KIBIBYTE, ChrBank::ROM.switchable(C2)),
        ChrWindow::new(0x0C00, 0x0FFF, 1 * KIBIBYTE, ChrBank::ROM.switchable(C3)),
        ChrWindow::new(0x1000, 0x13FF, 1 * KIBIBYTE, ChrBank::ROM.switchable(C4)),
        ChrWindow::new(0x1400, 0x17FF, 1 * KIBIBYTE, ChrBank::ROM.switchable(C5)),
        ChrWindow::new(0x1800, 0x1BFF, 1 * KIBIBYTE, ChrBank::ROM.switchable(C6)),
        ChrWindow::new(0x1C00, 0x1FFF, 1 * KIBIBYTE, ChrBank::ROM.switchable(C7)),
    ])
    .name_table_mirrorings(&[
        NameTableMirroring::ONE_SCREEN_LEFT_BANK,
        NameTableMirroring::VERTICAL,
        NameTableMirroring::ONE_SCREEN_RIGHT_BANK,
        NameTableMirroring::HORIZONTAL,
    ])
    .build();

// Namco 340
// TODO: Untested! Need relevant ROMs to test against (everything is mapper 19 instead).
pub struct Mapper210_2;

impl Mapper for Mapper210_2 {
    fn write_register(&mut self, mem: &mut Memory, addr: CpuAddress, value: u8) {
        match *addr {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x7FFF => { /* Do nothing. */ }
            0x8000..=0x87FF => mem.set_chr_register(C0, value),
            0x8800..=0x8FFF => mem.set_chr_register(C1, value),
            0x9000..=0x97FF => mem.set_chr_register(C2, value),
            0x9800..=0x9FFF => mem.set_chr_register(C3, value),
            0xA000..=0xA7FF => mem.set_chr_register(C4, value),
            0xA800..=0xAFFF => mem.set_chr_register(C5, value),
            0xB000..=0xB7FF => mem.set_chr_register(C6, value),
            0xB800..=0xBFFF => mem.set_chr_register(C7, value),
            0xC000..=0xDFFF => { /* Do nothing. */ }
            0xE000..=0xE7FF => {
                let fields = splitbits!(min=u8, value, "mmpppppp");
                mem.set_name_table_mirroring(fields.m);
                mem.set_prg_register(P0, fields.p);
            }
            0xE800..=0xEFFF => mem.set_prg_register(P1, value & 0b0011_1111),
            0xF000..=0xF7FF => mem.set_prg_register(P2, value & 0b0011_1111),
            0xF800..=0xFFFF => { /* Do nothing. */ }
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}
