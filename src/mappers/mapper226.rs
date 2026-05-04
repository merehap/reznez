use crate::mapper::*;

const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(2048 * KIBIBYTE)
    .prg_rom_outer_bank_size(1024 * KIBIBYTE)
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::ABSENT),
        PrgWindow::new(0x8000, 0xFFFF, 32 * KIBIBYTE, PrgBank::ROM.switchable(P)),
    ])
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::ABSENT),
        PrgWindow::new(0x8000, 0xBFFF, 16 * KIBIBYTE, PrgBank::ROM.switchable(P)),
        PrgWindow::new(0xC000, 0xFFFF, 16 * KIBIBYTE, PrgBank::ROM.switchable(P)),
    ])
    .chr_rom_max_size(8 * KIBIBYTE)
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x1FFF, 8 * KIBIBYTE, ChrBank::ROM_OR_RAM.write_status(WS0)),
    ])
    .name_table_mirrorings(&[
        NameTableMirroring::HORIZONTAL,
        NameTableMirroring::VERTICAL,
    ])
    .build();

// 76-in-1 and other multicarts
pub struct Mapper226;

impl Mapper for Mapper226 {
    fn write_register(&mut self, bus: &mut Bus, addr: CpuAddress, value: u8) {
        match *addr & 0x8001 {
            0x8000 => {
                let (prg_bank, mirroring, layout) = splitbits_named!(min=u8, value, "pmlp pppp");
                bus.set_prg_register(P, prg_bank);
                bus.set_name_table_mirroring(mirroring);
                bus.set_prg_layout(layout);
            }
            0x8001 => {
                let fields = splitbits!(value, ".... ..do");
                bus.set_writes_enabled(WS0, !fields.d);
                bus.set_prg_rom_outer_bank_number(fields.o as u8);
            }
            _ => { /* No regs here. */ }
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}