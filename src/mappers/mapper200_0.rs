use crate::mapper::*;

const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(256 * KIBIBYTE)
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::EMPTY),
        // Mirrored.
        PrgWindow::new(0x8000, 0xBFFF, 16 * KIBIBYTE, PrgBank::ROM.switchable(P0)),
        PrgWindow::new(0xC000, 0xFFFF, 16 * KIBIBYTE, PrgBank::ROM.switchable(P0)),
    ])
    .chr_rom_max_size(128 * KIBIBYTE)
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x1FFF, 8 * KIBIBYTE, ChrBank::ROM.switchable(C0)),
    ])
    .name_table_mirrorings(&[
        NameTableMirroring::VERTICAL,
        NameTableMirroring::HORIZONTAL,
    ])
    .build();

// NROM-128 multicarts with 16 PRG/CHR banks
pub struct Mapper200_0;

impl Mapper for Mapper200_0 {
    fn write_register(&mut self, mem: &mut Memory, addr: CpuAddress, _value: u8) {
        match *addr {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x7FFF => { /* Do nothing. */ }
            0x8000..=0xFFFF => {
                let bank_number = *addr & 0x000F;
                mem.set_prg_register(P0, bank_number);
                mem.set_chr_register(C0, bank_number);
                mem.set_name_table_mirroring((*addr >> 3) as u8 & 1);
            }
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}
