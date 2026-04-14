use crate::mapper::*;

const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(64 * KIBIBYTE)
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::ROM.switchable(P)),
        PrgWindow::new(0x8000, 0xFFFF, 32 * KIBIBYTE, PrgBank::ROM.fixed_number(2))
    ])
    .chr_rom_max_size(8 * KIBIBYTE)
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x1FFF, 8 * KIBIBYTE, ChrBank::ROM_OR_RAM),
    ])
    .fixed_name_table_mirroring()
    .build();

// Whirlwind Manu LH15 (FDS Conversions)
// TODO: Test (no test ROM readily available)
pub struct Mapper120;

impl Mapper for Mapper120 {
    fn write_register(&mut self, bus: &mut Bus, addr: CpuAddress, value: u8) {
        if *addr & 0xE100 == 0x41FF {
            bus.set_prg_register(P, value & 0b111);
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}