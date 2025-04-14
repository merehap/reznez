use crate::mapper::*;

const LAYOUT: Layout = Layout::builder()
    // Oversize definition?
    .prg_rom_max_size(2024 * KIBIBYTE)
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF, 8 * KIBIBYTE, PrgBank::EMPTY),
        PrgWindow::new(0x8000, 0x9FFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P0)),
        PrgWindow::new(0xA000, 0xBFFF, 8 * KIBIBYTE, PrgBank::ROM.fixed_index(-3)),
        PrgWindow::new(0xC000, 0xDFFF, 8 * KIBIBYTE, PrgBank::ROM.fixed_index(-2)),
        PrgWindow::new(0xE000, 0xFFFF, 8 * KIBIBYTE, PrgBank::ROM.fixed_index(-1)),
    ])
    .chr_rom_max_size(256 * KIBIBYTE)
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x0FFF, 4 * KIBIBYTE, ChrBank::RAM.switchable(C0)),
        ChrWindow::new(0x1000, 0x17FF, 2 * KIBIBYTE, ChrBank::RAM.switchable(C1)),
        ChrWindow::new(0x1800, 0x1FFF, 2 * KIBIBYTE, ChrBank::RAM.switchable(C2)),
    ])
    .do_not_align_large_chr_windows()
    .name_table_mirrorings(&[
        NameTableMirroring::VERTICAL,
        NameTableMirroring::HORIZONTAL,
    ])
    .build();

// NTDEC's TC-112
pub struct Mapper193;

impl Mapper for Mapper193 {
    fn write_to_cartridge_space(&mut self, params: &mut MapperParams, cpu_address: u16, value: u8) {
        match cpu_address & 0xE007 {
            0x6000 => params.set_chr_register(C0, value >> 1),
            0x6001 => params.set_chr_register(C1, value >> 1),
            0x6002 => params.set_chr_register(C2, value >> 1),
            0x6003 => params.set_prg_register(P0, value),
            0x6004 => params.set_name_table_mirroring(value & 1),
            _ => { /* Do nothing. */ }
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}
