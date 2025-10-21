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
    .chr_rom_max_size(128 * KIBIBYTE)
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x07FF, 2 * KIBIBYTE, ChrBank::ROM.switchable(C0)),
        ChrWindow::new(0x0800, 0x0FFF, 2 * KIBIBYTE, ChrBank::ROM.switchable(C1)),
        ChrWindow::new(0x1000, 0x17FF, 2 * KIBIBYTE, ChrBank::ROM.switchable(C2)),
        ChrWindow::new(0x1800, 0x1FFF, 2 * KIBIBYTE, ChrBank::ROM.switchable(C3)),
    ])
    .fixed_name_table_mirroring()
    .build();

use RegId::{Chr, Prg};
const BANK_NUMBER_REGISTER_IDS: [Option<RegId>; 8] =
    [None, None, Some(Chr(C0)), Some(Chr(C1)), Some(Chr(C2)), Some(Chr(C3)), Some(Prg(P0)), Some(Prg(P1))];

// NAMCOT-3446 
// Similar to Namcot 108, but with only large CHR windows and more PRG and CHR.
pub struct Mapper076 {
    selected_register_id: RegId,
}

impl Mapper for Mapper076 {
    fn write_register(&mut self, mem: &mut Memory, addr: CpuAddress, value: u8) {
        match *addr {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x7FFF => { /* Do nothing. */ }
            0x8000..=0x9FFF => {
                if addr.is_multiple_of(2) {
                    self.bank_select(mem, value);
                } else {
                    self.set_bank_number(mem, value);
                }
            }
            0xA000..=0xFFFF => { /* Do nothing. */ }
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}

impl Mapper076 {
    pub fn new() -> Self {
        Self {
            selected_register_id: Chr(C0),
        }
    }

    fn bank_select(&mut self, _mem: &mut Memory, value: u8) {
        if let Some(reg_id) = BANK_NUMBER_REGISTER_IDS[(value & 0b0000_0111) as usize] {
            self.selected_register_id = reg_id;
        }
    }

    fn set_bank_number(&mut self, mem: &mut Memory, value: u8) {
        let bank_number = u16::from(value & 0b0011_1111);
        match self.selected_register_id {
            Chr(cx) => mem.set_chr_register(cx, bank_number),
            Prg(px) => mem.set_prg_register(px, bank_number),
        }
    }
}

#[derive(Clone, Copy)]
enum RegId {
    Chr(ChrBankRegisterId),
    Prg(PrgBankRegisterId),
}