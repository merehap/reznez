use crate::mapper::*;

const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(512 * KIBIBYTE)
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::ABSENT),
        PrgWindow::new(0x8000, 0xFFFF, 32 * KIBIBYTE, PrgBank::ROM.switchable(P0)),
    ])
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::ABSENT),
        PrgWindow::new(0x8000, 0xBFFF, 16 * KIBIBYTE, PrgBank::ROM.switchable(P0)),
        PrgWindow::new(0xC000, 0xFFFF, 16 * KIBIBYTE, PrgBank::ROM.switchable(P0)),
    ])
    .chr_rom_max_size(128 * KIBIBYTE)
    // NTDEC 0324 has switchable ROM, GS-2017 has fixed RAM.
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x1FFF, 8 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C0)),
    ])
    .name_table_mirrorings(&[
        NameTableMirroring::VERTICAL,
        NameTableMirroring::HORIZONTAL,
    ])
    .build();

// NTDEC 0324 and GS-2017
pub struct Mapper061;

impl Mapper for Mapper061 {
    fn write_register(&mut self, bus: &mut Bus, addr: CpuAddress, _value: u8) {
        match *addr {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x7FFF => { /* Do nothing. */ }
            0x8000..=0xFFFF => {
                let fields = splitbits!(min=u8, *addr, ".... cccc m.ql pppp");
                let prg_index = combinebits!(fields.p, fields.q, "000p pppq");

                bus.set_prg_register(P0, prg_index);
                bus.set_chr_register(C0, fields.c);
                bus.set_name_table_mirroring(fields.m);
                bus.set_prg_layout(fields.l);
            }
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}