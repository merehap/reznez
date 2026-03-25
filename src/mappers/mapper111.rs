use crate::mapper::*;

const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(512 * KIBIBYTE)
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::ABSENT),
        PrgWindow::new(0x8000, 0xFFFF, 32 * KIBIBYTE, PrgBank::ROM.switchable(P)),
    ])
    .chr_rom_max_size(0 * KIBIBYTE)
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x1FFF, 8 * KIBIBYTE, ChrBank::RAM.switchable(C)),
        // 0x3F00 through 0x3FFF is still mapped to Palette RAM, despite this.
        ChrWindow::new(0x2000, 0x3FFF, 8 * KIBIBYTE, ChrBank::RAM.switchable(D)),
    ])
    .fixed_name_table_mirroring()
    .build();

// Cheapocabra or GTROM
pub struct Mapper111;

impl Mapper for Mapper111 {
    fn write_register(&mut self, bus: &mut Bus, addr: CpuAddress, value: u8) {
        if matches!(*addr, 0x5000..=0x5FFF | 0x7000..=0x7FFF) {
            let fields = splitbits!(min=u8, value, "grdc pppp");
            bus.set_chr_register(D, fields.d + 2); // CHR RAM pages 2 and 3
            bus.set_chr_register(C, fields.c);     // CHR RAM pages 0 and 1
            bus.set_prg_register(P, fields.p);
            // TODO: Green and red LEDs
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}