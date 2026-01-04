use crate::mapper::*;

const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(128 * KIBIBYTE)
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF, 8 * KIBIBYTE, PrgBank::ABSENT),
        PrgWindow::new(0x8000, 0x9FFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P0)),
        PrgWindow::new(0xA000, 0xBFFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P1)),
        PrgWindow::new(0xC000, 0xDFFF, 8 * KIBIBYTE, PrgBank::ROM.fixed_index(-2)),
        PrgWindow::new(0xE000, 0xFFFF, 8 * KIBIBYTE, PrgBank::ROM.fixed_index(-1)),
    ])
    .chr_rom_max_size(256 * KIBIBYTE)
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x07FF, 2 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C0)),
        ChrWindow::new(0x0800, 0x0FFF, 2 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C1)),
        ChrWindow::new(0x1000, 0x13FF, 1 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C2)),
        ChrWindow::new(0x1400, 0x17FF, 1 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C3)),
        ChrWindow::new(0x1800, 0x1BFF, 1 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C4)),
        ChrWindow::new(0x1C00, 0x1FFF, 1 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C5)),
    ])
    .name_table_mirrorings(&[
        NameTableMirroring::VERTICAL,
        NameTableMirroring::HORIZONTAL,
    ])
    .build();

use RegId::{Chr, Prg};
const BANK_REGISTER_IDS: [RegId; 8] = [Prg(P0), Prg(P1), Chr(C0), Chr(C1), Chr(C2), Chr(C3), Chr(C4), Chr(C5)];

// Huang Di and San Guo Zhi - Qun Xiong Zheng Ba
// Similar to mapper 206.
// FIXME: Currently jams, possibly due to broken DMC implementation.
pub struct Mapper112 {
    selected_register_id: RegId,
}

impl Mapper for Mapper112 {
    fn write_register(&mut self, bus: &mut Bus, addr: CpuAddress, value: u8) {
        match *addr & 0xE001 {
            0x8000 => self.selected_register_id = BANK_REGISTER_IDS[value as usize & 0b11],
            0xA000 => {
                match self.selected_register_id {
                    Prg(px) => bus.set_prg_register(px, value),
                    Chr(cx) => bus.set_chr_register(cx, value),
                }
            }
            0xE000 => bus.set_name_table_mirroring(value & 1),
            _ => { /* Do nothing. */ }
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}

impl Mapper112 {
    pub fn new() -> Self {
        Self { selected_register_id: Prg(P0) }
    }
}

#[derive(Clone, Copy)]
enum RegId {
    Prg(PrgBankRegisterId),
    Chr(ChrBankRegisterId),
}