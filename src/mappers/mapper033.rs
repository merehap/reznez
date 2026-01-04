use crate::mapper::*;

const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(512 * KIBIBYTE)
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF, 8 * KIBIBYTE, PrgBank::ABSENT),
        PrgWindow::new(0x8000, 0x9FFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P0)),
        PrgWindow::new(0xA000, 0xBFFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P1)),
        PrgWindow::new(0xC000, 0xDFFF, 8 * KIBIBYTE, PrgBank::ROM.fixed_index(-2)),
        PrgWindow::new(0xE000, 0xFFFF, 8 * KIBIBYTE, PrgBank::ROM.fixed_index(-1)),
    ])
    .chr_rom_max_size(512 * KIBIBYTE)
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x07FF, 2 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C0)),
        ChrWindow::new(0x0800, 0x0FFF, 2 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C1)),
        ChrWindow::new(0x1000, 0x13FF, 1 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C2)),
        ChrWindow::new(0x1400, 0x17FF, 1 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C3)),
        ChrWindow::new(0x1800, 0x1BFF, 1 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C4)),
        ChrWindow::new(0x1C00, 0x1FFF, 1 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C5)),
    ])
    .name_table_mirrorings(&[
        NameTableMirroring::VERTICAL,
        NameTableMirroring::HORIZONTAL,
    ])
    .build();

// Taito's TC0190
pub struct Mapper033;

impl Mapper for Mapper033 {
    fn write_register(&mut self, bus: &mut Bus, addr: CpuAddress, value: u8) {
        match *addr {
            0x0000..=0x401F => unreachable!(),
            0x8000 => {
                let fields = splitbits!(min=u8, value, ".mpppppp");
                bus.set_name_table_mirroring(fields.m);
                bus.set_prg_register(P0, fields.p);
            }
            0x8001 => bus.set_prg_register(P1, value & 0b0011_1111),
            // Large CHR windows: this allows accessing 512KiB CHR by doubling the bank indexes.
            0x8002 => bus.set_chr_register(C0, 2 * u16::from(value)),
            0x8003 => bus.set_chr_register(C1, 2 * u16::from(value)),
            // Small CHR windows.
            0xA000 => bus.set_chr_register(C2, value),
            0xA001 => bus.set_chr_register(C3, value),
            0xA002 => bus.set_chr_register(C4, value),
            0xA003 => bus.set_chr_register(C5, value),
            _ => { /* Do nothing. */ }
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}
