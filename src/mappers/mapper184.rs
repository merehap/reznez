use crate::mapper::*;

const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(32 * KIBIBYTE)
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::ABSENT),
        PrgWindow::new(0x8000, 0xFFFF, 32 * KIBIBYTE, PrgBank::ROM.fixed_number(0)),
    ])
    .chr_rom_max_size(64 * KIBIBYTE)
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x0FFF, 4 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C0)),
        ChrWindow::new(0x1000, 0x1FFF, 4 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C1)),
    ])
    .fixed_name_table_mirroring()
    .build();

// Sunsoft-1
pub struct Mapper184;

impl Mapper for Mapper184 {
    fn write_register(&mut self, bus: &mut Bus, addr: CpuAddress, value: u8) {
        match *addr {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x5FFF => { /* Do nothing. */ }
            0x6000..=0x7FFF => {
                let (high_bank, low_bank) = splitbits_named!(value, ".hhh.lll");
                bus.set_chr_register(C0, low_bank);
                bus.set_chr_register(C1, high_bank);
            }
            0x8000..=0xFFFF => { /* Do nothing. */ }
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}
