use crate::memory::mapper::*;

const LAYOUT: Layout = Layout::builder()
    .prg_max_size(128 * KIBIBYTE)
    .chr_max_size(128 * KIBIBYTE)
    .name_table_mirroring_source(NameTableMirroringSource::Cartridge)
    .prg_layouts(&[
        PRG_LAYOUT
    ])
    .chr_layouts(&[
        CHR_LAYOUT
    ])
    .build();

pub const PRG_LAYOUT: PrgLayout = PrgLayout::new(&[
    PrgWindow::new(0x6000, 0x7FFF, 8 * KIBIBYTE, Bank::EMPTY),
    PrgWindow::new(0x8000, 0x9FFF, 8 * KIBIBYTE, Bank::switchable_rom(P0)),
    PrgWindow::new(0xA000, 0xBFFF, 8 * KIBIBYTE, Bank::switchable_rom(P1)),
    PrgWindow::new(0xC000, 0xDFFF, 8 * KIBIBYTE, Bank::fixed_rom(BankIndex::SECOND_LAST)),
    PrgWindow::new(0xE000, 0xFFFF, 8 * KIBIBYTE, Bank::fixed_rom(BankIndex::LAST)),
]);

pub const CHR_LAYOUT: ChrLayout = ChrLayout::new(&[
    ChrWindow::new(0x0000, 0x07FF, 2 * KIBIBYTE, Bank::switchable_rom(C0)),
    ChrWindow::new(0x0800, 0x0FFF, 2 * KIBIBYTE, Bank::switchable_rom(C1)),
    ChrWindow::new(0x1000, 0x13FF, 1 * KIBIBYTE, Bank::switchable_rom(C2)),
    ChrWindow::new(0x1400, 0x17FF, 1 * KIBIBYTE, Bank::switchable_rom(C3)),
    ChrWindow::new(0x1800, 0x1BFF, 1 * KIBIBYTE, Bank::switchable_rom(C4)),
    ChrWindow::new(0x1C00, 0x1FFF, 1 * KIBIBYTE, Bank::switchable_rom(C5)),
]);

const BANK_INDEX_REGISTER_IDS: [BankRegisterId; 8] = [C0, C1, C2, C3, C4, C5, P0, P1];

// Similar to Mapper206, but allows up to 128KiB of CHR,
// and selects the second half of CHR for C2, C3, C4, and C5 for over-sized CHR.
pub struct Mapper088 {
    selected_register_id: BankRegisterId,
    extended_chr_present: bool,
}

impl Mapper for Mapper088 {
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

impl Mapper088 {
    pub fn new(cartridge: &Cartridge) -> Self {
        Self {
            selected_register_id: C0,
            extended_chr_present: cartridge.chr_rom().len() > 64 * KIBIBYTE as usize,
        }
    }

    fn bank_select(&mut self, _params: &mut MapperParams, value: u8) {
        self.selected_register_id = BANK_INDEX_REGISTER_IDS[(value & 0b0000_0111) as usize];
    }

    fn set_bank_index(&mut self, params: &mut MapperParams, value: u8) {
        let bank_index = match self.selected_register_id {
            // Double-width windows can only use even banks.
            C0 | C1 => value & 0b0011_1110,
            // Use the second 64KiB chunk of CHR.
            C2 | C3 | C4 | C5 if self.extended_chr_present => (value & 0b0011_1111) | 0b0100_0000,
            C2 | C3 | C4 | C5 => value & 0b0011_1111,
            P0 | P1 => value & 0b0000_1111,
            _ => unreachable!(
                "Bank Index Register ID {:?} is not used by this mapper.",
                self.selected_register_id
            ),
        };

        params.set_bank_register(self.selected_register_id, bank_index);
    }
}
