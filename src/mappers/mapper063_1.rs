
use crate::mapper::*;

const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(2048 * KIBIBYTE)
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::ABSENT),
        PrgWindow::new(0x8000, 0xFFFF, 32 * KIBIBYTE, PrgBank::ROM.switchable(P0)),
    ])
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::ABSENT),
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
        ReadWriteStatus::ReadWrite,
        ReadWriteStatus::ReadOnly,
    ])
    .build();

// 82AB
// Same as submapper 1, except there's one less PRG bank bit, and the RAM status bit is moved over
// to take its place.
// TODO: Untested. Test ROM needed.
pub struct Mapper063_1;

impl Mapper for Mapper063_1 {
    fn write_register(&mut self, mem: &mut Memory, addr: CpuAddress, _value: u8) {
        match *addr {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x7FFF => { /* Do nothing. */ }
            0x8000..=0xFFFF => {
                let fields = splitbits!(min=u8, *addr, ".... ..rp pppp pplm");
                mem.set_read_write_status(S0, fields.r);
                mem.set_prg_register(P0, fields.p);
                mem.set_prg_layout(fields.l);
                mem.set_name_table_mirroring(fields.m);
            }
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}
