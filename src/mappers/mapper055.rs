use crate::mapper::*;

const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(64 * KIBIBYTE)
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x67FF,  2 * KIBIBYTE, PrgBank::ROM.fixed_index(2)),
        PrgWindow::new(0x6800, 0x6FFF,  2 * KIBIBYTE, PrgBank::ROM.fixed_index(2)),
        PrgWindow::new(0x7000, 0x77FF,  2 * KIBIBYTE, PrgBank::ROM.fixed_index(3)),
        PrgWindow::new(0x7800, 0x7FFF,  2 * KIBIBYTE, PrgBank::ROM.fixed_index(3)),
        PrgWindow::new(0x8000, 0xBFFF, 16 * KIBIBYTE, PrgBank::ROM.fixed_index(0)),
        PrgWindow::new(0xC000, 0xFFFF, 16 * KIBIBYTE, PrgBank::ROM.fixed_index(1)),
    ])
    .chr_rom_max_size(8 * KIBIBYTE)
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x1FFF, 8 * KIBIBYTE, ChrBank::ROM_OR_RAM.fixed_index(0)),
    ])
    .fixed_name_table_mirroring()
    .build();

// BTL-MARIO1-MALEE2
pub struct Mapper055;

impl Mapper for Mapper055 {
    fn write_register(&mut self, _mem: &mut Memory, addr: CpuAddress, _value: u8) {
        match *addr {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0xFFFF => { /* Do nothing here, just like NROM. */ }
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}
