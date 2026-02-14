use crate::mapper::*;

const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(512 * KIBIBYTE)
    .prg_layout(&[
        // FIXME: This is supposed to be 2KiBs mirrored. Family Circuit doesn't work without it.
        PrgWindow::new(0x6000, 0x7FFF, 8 * KIBIBYTE, PrgBank::RAM_OR_ABSENT),
        PrgWindow::new(0x8000, 0x9FFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P)),
        PrgWindow::new(0xA000, 0xBFFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(Q)),
        PrgWindow::new(0xC000, 0xDFFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(R)),
        PrgWindow::new(0xE000, 0xFFFF, 8 * KIBIBYTE, PrgBank::ROM.fixed_number(-1)),
    ])
    .chr_rom_max_size(256 * KIBIBYTE)
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x03FF, 1 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C)),
        ChrWindow::new(0x0400, 0x07FF, 1 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(D)),
        ChrWindow::new(0x0800, 0x0BFF, 1 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(E)),
        ChrWindow::new(0x0C00, 0x0FFF, 1 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(F)),
        ChrWindow::new(0x1000, 0x13FF, 1 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(G)),
        ChrWindow::new(0x1400, 0x17FF, 1 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(H)),
        ChrWindow::new(0x1800, 0x1BFF, 1 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(I)),
        ChrWindow::new(0x1C00, 0x1FFF, 1 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(J)),
    ])
    .fixed_name_table_mirroring()
    .build();

// Namco 175
pub struct Mapper210_1;

impl Mapper for Mapper210_1 {
    fn write_register(&mut self, bus: &mut Bus, addr: CpuAddress, value: u8) {
        match *addr {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x7FFF => { /* Do nothing. */ }
            0x8000..=0x87FF => bus.set_chr_register(C, value),
            0x8800..=0x8FFF => bus.set_chr_register(D, value),
            0x9000..=0x97FF => bus.set_chr_register(E, value),
            0x9800..=0x9FFF => bus.set_chr_register(F, value),
            0xA000..=0xA7FF => bus.set_chr_register(G, value),
            0xA800..=0xAFFF => bus.set_chr_register(H, value),
            0xB000..=0xB7FF => bus.set_chr_register(I, value),
            0xB800..=0xBFFF => bus.set_chr_register(J, value),
            0xC000..=0xC7FF => { /* TODO: External PRG RAM enable. */ }
            0xC800..=0xDFFF => { /* Do nothing. */ }
            0xE000..=0xE7FF => bus.set_prg_register(P, value & 0b0011_1111),
            0xE800..=0xEFFF => bus.set_prg_register(Q, value & 0b0011_1111),
            0xF000..=0xF7FF => bus.set_prg_register(R, value & 0b0011_1111),
            0xF800..=0xFFFF => { /* Do nothing. */ }
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}
