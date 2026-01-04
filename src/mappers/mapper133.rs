use crate::mapper::*;

const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(256 * KIBIBYTE)
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::ABSENT),
        PrgWindow::new(0x8000, 0xFFFF, 32 * KIBIBYTE, PrgBank::ROM.switchable(P0)),
    ])
    .chr_rom_max_size(128 * KIBIBYTE)
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x1FFF, 8 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C0)),
    ])
    .fixed_name_table_mirroring()
    .build();

// Sachen 3009
pub struct Mapper133;

impl Mapper for Mapper133 {
    fn write_register(&mut self, bus: &mut Bus, addr: CpuAddress, value: u8) {
        match *addr & 0xE100 {
            0x0000..=0x401F => unreachable!(),
            0x4100 => {
                let banks = splitbits!(value, ".....pcc");
                bus.set_prg_register(P0, banks.p as u8);
                bus.set_chr_register(C0, banks.c);
            }
            _ => { /* Do nothing. */ }
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}
