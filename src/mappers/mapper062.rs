use crate::mapper::*;

const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(4096 * KIBIBYTE)
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::EMPTY),
        PrgWindow::new(0x8000, 0xFFFF, 32 * KIBIBYTE, PrgBank::ROM.switchable(P0)),
    ])
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::EMPTY),
        PrgWindow::new(0x8000, 0xBFFF, 16 * KIBIBYTE, PrgBank::ROM.switchable(P0)),
        PrgWindow::new(0xC000, 0xFFFF, 16 * KIBIBYTE, PrgBank::ROM.switchable(P0)),
    ])
    .chr_rom_max_size(1024 * KIBIBYTE)
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x1FFF, 8 * KIBIBYTE, ChrBank::ROM.switchable(C0)),
    ])
    .name_table_mirrorings(&[
        NameTableMirroring::VERTICAL,
        NameTableMirroring::HORIZONTAL,
    ])
    .build();

// Super 700-in-1
pub struct Mapper062; 

impl Mapper for Mapper062 {
    fn write_register(&mut self, mem: &mut Memory, addr: CpuAddress, value: u8) {
        match *addr {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x7FFF => { /* Do nothing. */ }
            0x8000..=0xFFFF => {
                let fields = splitbits!(min=u8, *addr, "..pp pppp mqlc cccc");
                mem.set_name_table_mirroring(fields.m);
                mem.set_prg_layout(fields.l);

                let prg_index = combinebits!(fields.q, fields.p, "0qpppppp");
                mem.set_prg_register(P0, prg_index);
                let chr_index = (fields.c << 2) | (value & 0b11);
                mem.set_chr_register(C0, chr_index);
            }
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}
