use crate::mapper::*;

const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(4096 * KIBIBYTE)
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::EMPTY),
        PrgWindow::new(0x8000, 0xBFFF, 16 * KIBIBYTE, PrgBank::ROM.fixed_index(0)),
        PrgWindow::new(0xC000, 0xFFFF, 16 * KIBIBYTE, PrgBank::ROM.switchable(P0)),
    ])
    .chr_rom_max_size(8 * KIBIBYTE)
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x1FFF, 8 * KIBIBYTE, ChrBank::ROM.fixed_index(0)),
    ])
    .build();

// UNROM, but the fixed bank and the switchable bank are swapped.
pub struct Mapper180;

impl Mapper for Mapper180 {
    fn write_register(&mut self, params: &mut MapperParams, cpu_address: u16, value: u8) {
        match cpu_address {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x7FFF => { /* Do nothing. */ },
            0x8000..=0xFFFF => params.set_prg_register(P0, value & 0b0000_0111),
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}
