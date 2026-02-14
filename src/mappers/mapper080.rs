use crate::mapper::*;

pub const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(2048 * KIBIBYTE)
    .prg_layout(PRG_LAYOUT)
    .chr_rom_max_size(256 * KIBIBYTE)
    .chr_layout(CHR_LAYOUT)
    .name_table_mirrorings(&[
        NameTableMirroring::HORIZONTAL,
        NameTableMirroring::VERTICAL,
    ])
    .build();

pub const PRG_LAYOUT: &[PrgWindow] = &[
    PrgWindow::new(0x6000, 0x7EFF, 7 * KIBIBYTE + 3 * KIBIBYTE / 4, PrgBank::ABSENT),
    PrgWindow::new(0x7F00, 0x7F7F, KIBIBYTE / 8, PrgBank::RAM_OR_ABSENT.fixed_number(0).read_write_status(RS0, WS0)),
    PrgWindow::new(0x7F80, 0x7FFF, KIBIBYTE / 8, PrgBank::RAM_OR_ABSENT.fixed_number(0).read_write_status(RS0, WS0)),
    PrgWindow::new(0x8000, 0x9FFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P)),
    PrgWindow::new(0xA000, 0xBFFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(Q)),
    PrgWindow::new(0xC000, 0xDFFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(R)),
    PrgWindow::new(0xE000, 0xFFFF, 8 * KIBIBYTE, PrgBank::ROM.fixed_number(-1)),
];
pub const CHR_LAYOUT: &[ChrWindow] = &[
    ChrWindow::new(0x0000, 0x07FF, 2 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C)),
    ChrWindow::new(0x0800, 0x0FFF, 2 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(D)),
    ChrWindow::new(0x1000, 0x13FF, 1 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(E)),
    ChrWindow::new(0x1400, 0x17FF, 1 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(F)),
    ChrWindow::new(0x1800, 0x1BFF, 1 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(G)),
    ChrWindow::new(0x1C00, 0x1FFF, 1 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(H)),
];

// Taito's X1-005
pub struct Mapper080;

impl Mapper for Mapper080 {
    fn write_register(&mut self, bus: &mut Bus, addr: CpuAddress, value: u8) {
        match *addr {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x7EEF => { /* Do nothing. */ }
            0x7EF0 => bus.set_chr_register(C, value),
            0x7EF1 => bus.set_chr_register(D, value),
            0x7EF2 => bus.set_chr_register(E, value),
            0x7EF3 => bus.set_chr_register(F, value),
            0x7EF4 => bus.set_chr_register(G, value),
            0x7EF5 => bus.set_chr_register(H, value),
            0x7EF6..=0x7EF7 => bus.set_name_table_mirroring(value & 1),
            0x7EF8..=0x7EF9 => {
                let ram_enabled = value == 0xA3;
                bus.set_reads_enabled(RS0, ram_enabled);
                bus.set_writes_enabled(WS0, ram_enabled);
            }
            0x7EFA..=0x7EFB => bus.set_prg_register(P, value),
            0x7EFC..=0x7EFD => bus.set_prg_register(Q, value),
            0x7EFE..=0x7EFF => bus.set_prg_register(R, value),
            0x7F00..=0xFFFF => { /* Do nothing. */ }
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}
