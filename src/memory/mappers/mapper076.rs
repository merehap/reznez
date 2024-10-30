use crate::memory::mapper::*;

const LAYOUT: Layout = Layout::builder()
    .prg_max_size(128 * KIBIBYTE)
    .chr_max_size(128 * KIBIBYTE)
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF, 8 * KIBIBYTE, Bank::EMPTY),
        PrgWindow::new(0x8000, 0x9FFF, 8 * KIBIBYTE, Bank::switchable_rom(P0)),
        PrgWindow::new(0xA000, 0xBFFF, 8 * KIBIBYTE, Bank::switchable_rom(P1)),
        PrgWindow::new(0xC000, 0xDFFF, 8 * KIBIBYTE, Bank::fixed_rom(BankIndex::SECOND_LAST)),
        PrgWindow::new(0xE000, 0xFFFF, 8 * KIBIBYTE, Bank::fixed_rom(BankIndex::LAST)),
    ])
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x07FF, 2 * KIBIBYTE, Bank::switchable_rom(C0)),
        ChrWindow::new(0x0800, 0x0FFF, 2 * KIBIBYTE, Bank::switchable_rom(C1)),
        ChrWindow::new(0x1000, 0x17FF, 2 * KIBIBYTE, Bank::switchable_rom(C2)),
        ChrWindow::new(0x1800, 0x1FFF, 2 * KIBIBYTE, Bank::switchable_rom(C3)),
    ])
    .build();

const BANK_INDEX_REGISTER_IDS: [Option<BankRegisterId>; 8] =
    [None, None, Some(C0), Some(C1), Some(C2), Some(C3), Some(P0), Some(P1)];

// NAMCOT-3446 
// Similar to Namcot 108, but with only large CHR windows and more PRG and CHR.
pub struct Mapper076 {
    selected_register_id: BankRegisterId,
}

impl Mapper for Mapper076 {
    fn write_to_cartridge_space(&mut self, params: &mut MapperParams, address: CpuAddress, value: u8) {
        match address.to_raw() {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x5FFF => { /* Do nothing. */ }
            0x6000..=0x7FFF => params.write_prg(address, value),
            0x8000..=0x9FFF => {
                if address.to_raw() % 2 == 0 {
                    self.bank_select(params, value);
                } else {
                    self.set_bank_index(params, value);
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
            selected_register_id: C0,
        }
    }

    fn bank_select(&mut self, _params: &mut MapperParams, value: u8) {
        if let Some(reg_id) = BANK_INDEX_REGISTER_IDS[(value & 0b0000_0111) as usize] {
            self.selected_register_id = reg_id;
        }
    }

    fn set_bank_index(&mut self, params: &mut MapperParams, value: u8) {
        let bank_index = u16::from(value & 0b0011_1111);
        params.set_bank_register(self.selected_register_id, bank_index);
    }
}
