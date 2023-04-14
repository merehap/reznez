use crate::memory::mapper::*;

const PRG_LAYOUT: PrgLayout = PrgLayout::new(&[
    PrgWindow::new(0x6000, 0x7FFF, 8 * KIBIBYTE, PrgType::Empty),
    PrgWindow::new(0x8000, 0x9FFF, 8 * KIBIBYTE, PrgType::SwitchableBank(Rom, P0)),
    PrgWindow::new(0xA000, 0xBFFF, 8 * KIBIBYTE, PrgType::SwitchableBank(Rom, P1)),
    PrgWindow::new(0xC000, 0xDFFF, 8 * KIBIBYTE, PrgType::FixedBank(Rom, BankIndex::SECOND_LAST)),
    PrgWindow::new(0xE000, 0xFFFF, 8 * KIBIBYTE, PrgType::FixedBank(Rom, BankIndex::LAST)),
]);

const CHR_LAYOUT: ChrLayout = ChrLayout::new(&[
    ChrWindow::new(0x0000, 0x07FF, 2 * KIBIBYTE, ChrType::SwitchableBank(Rom, C0)),
    ChrWindow::new(0x0800, 0x0FFF, 2 * KIBIBYTE, ChrType::SwitchableBank(Rom, C1)),
    ChrWindow::new(0x1000, 0x13FF, 1 * KIBIBYTE, ChrType::SwitchableBank(Rom, C2)),
    ChrWindow::new(0x1400, 0x17FF, 1 * KIBIBYTE, ChrType::SwitchableBank(Rom, C3)),
    ChrWindow::new(0x1800, 0x1BFF, 1 * KIBIBYTE, ChrType::SwitchableBank(Rom, C4)),
    ChrWindow::new(0x1C00, 0x1FFF, 1 * KIBIBYTE, ChrType::SwitchableBank(Rom, C5)),
]);

const BANK_INDEX_REGISTER_IDS: [BankIndexRegisterId; 8] = [C0, C1, C2, C3, C4, C5, P0, P1];

// DxROM, Tengen MIMIC-1, Namco 118
// A much simpler predecessor to MMC3.
pub struct Mapper206 {
    selected_register_id: BankIndexRegisterId,
}

impl Mapper for Mapper206 {
    fn initial_layout(&self) -> InitialLayout {
        InitialLayout::builder()
            .prg_max_bank_count(16)
            .prg_bank_size(8 * KIBIBYTE)
            .prg_windows(PRG_LAYOUT)
            .chr_max_bank_count(64)
            .chr_bank_size(1 * KIBIBYTE)
            .chr_windows(CHR_LAYOUT)
            .name_table_mirroring_source(NameTableMirroringSource::Cartridge)
            .build()
    }

    fn write_to_cartridge_space(&mut self, params: &mut MapperParams, address: CpuAddress, value: u8) {
        match address.to_raw() {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x5FFF => { /* Do nothing. */ }
            0x6000..=0x7FFF => params.write_prg(address, value),
            0x8000..=0x9FFF => {
                if address.to_raw() % 2 == 0 {
                    self.bank_select(params, value)
                } else {
                    self.set_bank_index(params, value)
                }
            }
            0xA000..=0xFFFF => { /* Do nothing. */ }
        }
    }
}

impl Mapper206 {
    pub fn new() -> Self {
        Self { selected_register_id: C0 }
    }

    fn bank_select(&mut self, _params: &mut MapperParams, value: u8) {
        self.selected_register_id = BANK_INDEX_REGISTER_IDS[(value & 0b0000_0111) as usize];
    }

    fn set_bank_index(&mut self, params: &mut MapperParams, value: u8) {
        let mask = match self.selected_register_id {
            // Double-width windows can only use even banks.
            C0 | C1 => 0b0011_1110,
            C2 | C3 | C4 | C5 => 0b0011_1111,
            P0 | P1 => 0b0000_1111,
            _ => unreachable!(
                "Bank Index Register ID {:?} is not used by this mapper.",
                self.selected_register_id
            ),
        };

        let bank_index = value & mask;
        params.set_bank_index_register(self.selected_register_id, bank_index);
    }
}
