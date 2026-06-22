use crate::mapper::*;

const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(256 * KIBIBYTE)
    .prg_rom_outer_bank_size(64 * KIBIBYTE)
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, Prg::ABSENT),
        PrgWindow::new(0x8000, 0xBFFF, 16 * KIBIBYTE, Prg::ROM).switchable(P),
        PrgWindow::new(0xC000, 0xFFFF, 16 * KIBIBYTE, Prg::ROM).fixed_number(-1),
    ])
    .chr_rom_max_size(8 * KIBIBYTE)
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x1FFF, 8 * KIBIBYTE, Chr::ROM_OR_RAM).fixed_index(0),
    ])
    .fixed_name_table_mirroring()
    .build();

// Camerica/Codemasters/Quattro
pub struct Mapper232;

impl Mapper for Mapper232 {
    fn write_register(&mut self, bus: &mut Bus, addr: CpuAddress, value: u8) {
        match *addr {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x7FFF => { /* Do nothing. */ }
            0x8000..=0xBFFF => bus.set_prg_rom_outer_bank_number((value >> 3) & 0b11),
            0xC000..=0xFFFF => bus.set_prg_register(P, value & 0b11),
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}
