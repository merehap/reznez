use crate::mapper::*;

const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(128 * KIBIBYTE)
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::ABSENT),
        PrgWindow::new(0x8000, 0xBFFF, 16 * KIBIBYTE, PrgBank::ROM.switchable(P0)),
        PrgWindow::new(0xC000, 0xFFFF, 16 * KIBIBYTE, PrgBank::ROM.switchable(P0)),
    ])
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::ABSENT),
        PrgWindow::new(0x8000, 0xFFFF, 32 * KIBIBYTE, PrgBank::ROM.switchable(P0)),
    ])
    .chr_rom_max_size(64 * KIBIBYTE)
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x1FFF, 8 * KIBIBYTE, ChrBank::ROM.switchable(C0))
    ])
    .name_table_mirrorings(&[
        NameTableMirroring::VERTICAL,
        NameTableMirroring::HORIZONTAL,
    ])
    .build();

// NTDec 5-in-1
// TODO: Test. Need a test ROM.
// TODO: Retain CPU-internal RAM contents upon RESET.
pub struct Mapper174;

impl Mapper for Mapper174 {
    fn write_register(&mut self, mem: &mut Memory, addr: CpuAddress, _value: u8) {
        match *addr {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x5FFF => { /* Do nothing. */ }
            0x6000..=0xFFFF => {
                let fields = splitbits!(min=u8, *addr, ".... .... lppp cccm");
                mem.set_prg_layout(fields.l);
                mem.set_prg_register(P0, if fields.l == 0 { fields.p } else { fields.p & 0b110 });
                mem.set_name_table_mirroring(fields.m);
            }
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}