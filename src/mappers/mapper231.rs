use crate::mapper::*;

const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(512 * KIBIBYTE)
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::ABSENT),
        PrgWindow::new(0x8000, 0xBFFF, 16 * KIBIBYTE, PrgBank::ROM.switchable(P0)),
        PrgWindow::new(0xC000, 0xFFFF, 16 * KIBIBYTE, PrgBank::ROM.switchable(P1)),
    ])
    .chr_rom_max_size(8 * KIBIBYTE)
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x1FFF, 8 * KIBIBYTE, ChrBank::ROM_OR_RAM.fixed_index(0))
    ])
    .name_table_mirrorings(&[
        NameTableMirroring::VERTICAL,
        NameTableMirroring::HORIZONTAL,
    ])
    .build();

// 20-in-1
pub struct Mapper231;

impl Mapper for Mapper231 {
    fn write_register(&mut self, bus: &mut Bus, addr: CpuAddress, _value: u8) {
        match *addr {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x7FFF => { /* No regs here. */ }
            0x8000..=0xFFFF => {
                let fields = splitbits!(min=u8, *addr, ".... .... m.lp ppp.");
                bus.set_name_table_mirroring(fields.m);
                bus.set_prg_register(P0, fields.p << 1);
                bus.set_prg_register(P1, (fields.p << 1) | fields.l);
            }
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}