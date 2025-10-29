use crate::mapper::*;

const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(128 * KIBIBYTE)
    .prg_layout(PRG_WINDOWS)
    .chr_rom_max_size(128 * KIBIBYTE)
    .chr_layout(CHR_WINDOWS)
    .fixed_name_table_mirroring()
    .build();

pub const PRG_WINDOWS: &[PrgWindow] = &[
    PrgWindow::new(0x6000, 0x7FFF, 8 * KIBIBYTE, PrgBank::ABSENT),
    PrgWindow::new(0x8000, 0x9FFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P0)),
    PrgWindow::new(0xA000, 0xBFFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P1)),
    PrgWindow::new(0xC000, 0xDFFF, 8 * KIBIBYTE, PrgBank::ROM.fixed_index(-2)),
    PrgWindow::new(0xE000, 0xFFFF, 8 * KIBIBYTE, PrgBank::ROM.fixed_index(-1)),
];

pub const CHR_WINDOWS: &[ChrWindow] = &[
    ChrWindow::new(0x0000, 0x07FF, 2 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C0)),
    ChrWindow::new(0x0800, 0x0FFF, 2 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C1)),
    ChrWindow::new(0x1000, 0x13FF, 1 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C2)),
    ChrWindow::new(0x1400, 0x17FF, 1 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C3)),
    ChrWindow::new(0x1800, 0x1BFF, 1 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C4)),
    ChrWindow::new(0x1C00, 0x1FFF, 1 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C5)),
];

use RegId::{Chr, Prg};
const BANK_NUMBER_REGISTER_IDS: [RegId; 8] = [Chr(C0), Chr(C1), Chr(C2), Chr(C3), Chr(C4), Chr(C5), Prg(P0), Prg(P1)];

// Similar to Mapper206, but allows up to 128KiB of CHR,
// and selects the second half of CHR for C2, C3, C4, and C5 for over-sized CHR.
pub struct Mapper088 {
    selected_register_id: RegId,
}

impl Mapper for Mapper088 {
    fn write_register(&mut self, mem: &mut Memory, addr: CpuAddress, value: u8) {
        match *addr {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x7FFF => { /* Do nothing. */ }
            0x8000..=0x9FFF if addr.is_multiple_of(2) => {
                self.selected_register_id = BANK_NUMBER_REGISTER_IDS[(value & 0b111) as usize];
            }
            0x8000..=0x9FFF => {
                match self.selected_register_id {
                    // Always use only the first 64KiB of CHR for the left pattern table.
                    Chr(id@(C0 | C1)) => mem.set_chr_register(id, value & 0b0011_1110),
                    // If it is available, use the second 64KiB half of CHR for the right pattern table.
                    Chr(id@(C2 | C3 | C4 | C5)) => mem.set_chr_register(id, (value & 0b0011_1111) | 0b0100_0000),
                    Prg(id@(P0 | P1)) => mem.set_prg_register(id, value & 0b0000_1111),
                    _ => unreachable!(),
                };
            }
            0xA000..=0xFFFF => { /* Do nothing. */ }
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}

impl Mapper088 {
    pub fn new() -> Self {
        Self { selected_register_id: Chr(C0) }
    }
}

#[derive(Clone, Copy, Debug)]
enum RegId {
    Chr(ChrBankRegisterId),
    Prg(PrgBankRegisterId),
}