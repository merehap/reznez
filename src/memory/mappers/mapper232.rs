use crate::memory::mapper::*;

const INITIAL_LAYOUT: InitialLayout = InitialLayout::builder()
    .prg_max_bank_count(16)
    .prg_bank_size(16 * KIBIBYTE)
    .prg_windows_by_board(&[(Board::Any, PRG_WINDOWS)])
    .chr_max_bank_count(1)
    .chr_bank_size(8 * KIBIBYTE)
    .chr_windows(CHR_WINDOWS)
    .name_table_mirroring_source(NameTableMirroringSource::Cartridge)
    // The last bank for any of the mapper 232 PRG "blocks".
    .override_bank_index_register(P1, BankIndex::from_u8(0b11))
    .build();

const PRG_WINDOWS: &[PrgWindow] = &[
    PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgType::Empty),
    PrgWindow::new(0x8000, 0xBFFF, 16 * KIBIBYTE, PrgType::VariableBank(Rom, P0)),
    PrgWindow::new(0xC000, 0xFFFF, 16 * KIBIBYTE, PrgType::VariableBank(Rom, P1)),
];

const CHR_WINDOWS: &[ChrWindow] = &[
    ChrWindow::new(0x0000, 0x1FFF, 8 * KIBIBYTE, ChrType::ConstantBank(Rom, BankIndex::FIRST)),
];

// Similar to mapper 71.
pub struct Mapper232 {
    params: MapperParams,
}

impl Mapper for Mapper232 {
    fn write_to_cartridge_space(&mut self, address: CpuAddress, value: u8) {
        let value = u16::from(value);
        match address.to_raw() {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x7FFF => { /* Do nothing. */ }
            0x8000..=0xBFFF => {
                let set_high_bank_bits = |bank_index| {
                    (bank_index & 0b0011) | ((value & 0b1_1000) >> 1)
                };
                self.prg_memory_mut().update_bank_index_register(P0, &set_high_bank_bits);
                self.prg_memory_mut().update_bank_index_register(P1, &set_high_bank_bits);
            }
            0xC000..=0xFFFF => {
                let set_low_bank_bits = |bank_index| {
                    (bank_index & 0b1100) | (value & 0b0011)
                };
                self.prg_memory_mut().update_bank_index_register(P0, &set_low_bank_bits);
            }
        }
    }

    fn params(&self) -> &MapperParams { &self.params }
    fn params_mut(&mut self) -> &mut MapperParams { &mut self.params }
}

impl Mapper232 {
    pub fn new(cartridge: &Cartridge) -> Result<Mapper232, String> {
        Ok(Mapper232 {
            params: INITIAL_LAYOUT.make_mapper_params(cartridge, Board::Any),
        })
    }
}
