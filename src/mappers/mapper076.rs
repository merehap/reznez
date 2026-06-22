use crate::mapper::*;

const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(128 * KIBIBYTE)
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF, 8 * KIBIBYTE, Prg::ABSENT),
        PrgWindow::new(0x8000, 0x9FFF, 8 * KIBIBYTE, Prg::ROM).switchable(P),
        PrgWindow::new(0xA000, 0xBFFF, 8 * KIBIBYTE, Prg::ROM).switchable(Q),
        PrgWindow::new(0xC000, 0xDFFF, 8 * KIBIBYTE, Prg::ROM).fixed_number(-2),
        PrgWindow::new(0xE000, 0xFFFF, 8 * KIBIBYTE, Prg::ROM).fixed_number(-1),
    ])
    .chr_rom_max_size(128 * KIBIBYTE)
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x07FF, 2 * KIBIBYTE, Chr::ROM_OR_RAM).switchable(C),
        ChrWindow::new(0x0800, 0x0FFF, 2 * KIBIBYTE, Chr::ROM_OR_RAM).switchable(D),
        ChrWindow::new(0x1000, 0x17FF, 2 * KIBIBYTE, Chr::ROM_OR_RAM).switchable(E),
        ChrWindow::new(0x1800, 0x1FFF, 2 * KIBIBYTE, Chr::ROM_OR_RAM).switchable(F),
    ])
    .fixed_name_table_mirroring()
    .build();

use RegId::{CHR, PRG};
const BANK_NUMBER_REGISTER_IDS: [Option<RegId>; 8] =
    [None, None, Some(CHR(C)), Some(CHR(D)), Some(CHR(E)), Some(CHR(F)), Some(PRG(P)), Some(PRG(Q))];

// NAMCOT-3446
// Similar to Namcot 108, but with only large CHR windows and more PRG and CHR.
pub struct Mapper076 {
    selected_register_id: RegId,
}

impl Mapper for Mapper076 {
    fn write_register(&mut self, bus: &mut Bus, addr: CpuAddress, value: u8) {
        match *addr {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x7FFF => { /* Do nothing. */ }
            0x8000..=0x9FFF => {
                if addr.is_multiple_of(2) {
                    self.bank_select(bus, value);
                } else {
                    self.set_bank_number(bus, value);
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
            selected_register_id: CHR(C),
        }
    }

    fn bank_select(&mut self, _bus: &mut Bus, value: u8) {
        if let Some(reg_id) = BANK_NUMBER_REGISTER_IDS[(value & 0b0000_0111) as usize] {
            self.selected_register_id = reg_id;
        }
    }

    fn set_bank_number(&mut self, bus: &mut Bus, value: u8) {
        let bank_number = u16::from(value & 0b0011_1111);
        match self.selected_register_id {
            CHR(cx) => bus.set_chr_register(cx, bank_number),
            PRG(px) => bus.set_prg_register(px, bank_number),
        }
    }
}

#[derive(Clone, Copy)]
enum RegId {
    CHR(ChrBankRegisterId),
    PRG(PrgBankRegisterId),
}
