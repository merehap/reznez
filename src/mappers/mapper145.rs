use crate::mapper::*;

const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(32 * KIBIBYTE)
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::EMPTY),
        PrgWindow::new(0x8000, 0xFFFF, 32 * KIBIBYTE, PrgBank::ROM.fixed_index(0)),
    ])
    .chr_rom_max_size(16 * KIBIBYTE)
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x1FFF, 8 * KIBIBYTE, ChrBank::ROM.switchable(C0)),
    ])
    .fixed_name_table_mirroring()
    .build();

// SA-72007 - Sidewinder
pub struct Mapper145;

impl Mapper for Mapper145 {
    fn write_register(&mut self, mem: &mut Memory, addr: CpuAddress, value: u8) {
        match *addr & 0xE100 {
            0x0000..=0x401F => unreachable!(),
            0x4100 => mem.set_chr_register(C0, splitbits_named!(value, "c.......")),
            _ => { /* Do nothing. */ }
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}
