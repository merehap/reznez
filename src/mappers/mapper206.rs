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
    .chr_rom_max_size(64 * KIBIBYTE)
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x07FF, 2 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C0)),
        ChrWindow::new(0x0800, 0x0FFF, 2 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C1)),
        ChrWindow::new(0x1000, 0x13FF, 1 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C2)),
        ChrWindow::new(0x1400, 0x17FF, 1 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C3)),
        ChrWindow::new(0x1800, 0x1BFF, 1 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C4)),
        ChrWindow::new(0x1C00, 0x1FFF, 1 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C5)),
    ])
    .fixed_name_table_mirroring()
    .build();

use RegId::{Chr, Prg};
const BANK_NUMBER_REGISTER_IDS: [RegId; 8] = [Chr(C0), Chr(C1), Chr(C2), Chr(C3), Chr(C4), Chr(C5), Prg(P0), Prg(P1)];

// DxROM, Tengen MIMIC-1, Namco 118
// A much simpler predecessor to MMC3.
pub struct Mapper206 {
    selected_register_id: RegId,
}

impl Mapper for Mapper206 {
    fn write_register(&mut self, bus: &mut Bus, addr: CpuAddress, value: u8) {
        match *addr {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x7FFF => { /* Do nothing. */ }
            0x8000..=0x9FFF if addr.is_multiple_of(2) => {
                self.selected_register_id = BANK_NUMBER_REGISTER_IDS[(value & 0b111) as usize];
            }
            0x8000..=0x9FFF => {
                match self.selected_register_id {
                    Chr(id) => bus.set_chr_register(id, value & 0b0011_1111),
                    Prg(id) => bus.set_prg_register(id, value & 0b0000_1111),
                }
            }
            0xA000..=0xFFFF => { /* Do nothing. */ }
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}

impl Mapper206 {
    pub fn new() -> Self {
        Self { selected_register_id: Chr(C0) }
    }
}

#[derive(Clone, Copy, Debug)]
enum RegId {
    Chr(ChrBankRegisterId),
    Prg(PrgBankRegisterId),
}