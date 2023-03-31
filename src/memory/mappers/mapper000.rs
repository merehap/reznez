use crate::memory::mapper::*;

const INITIAL_LAYOUT: InitialLayout = InitialLayout::builder()
    .prg_max_bank_count(1)
    .prg_bank_size(32 * KIBIBYTE)
    .prg_windows(PRG_WINDOWS)
    .chr_max_bank_count(1)
    .chr_bank_size(8 * KIBIBYTE)
    .chr_windows(CHR_WINDOWS)
    .name_table_mirroring_source(NameTableMirroringSource::Cartridge)
    .build();

const PRG_WINDOWS: PrgWindows = PrgWindows::new(&[
    PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgType::Empty),
    PrgWindow::new(0x8000, 0xFFFF, 32 * KIBIBYTE, PrgType::ConstantBank(Rom, BankIndex::FIRST)),
]);

const CHR_WINDOWS: ChrWindows = ChrWindows::new(&[
    ChrWindow::new(0x0000, 0x1FFF, 8 * KIBIBYTE, ChrType::ConstantBank(Rom, BankIndex::FIRST)),
]);

// NROM
pub struct Mapper000 {
    params: MapperParams,
}

impl Mapper for Mapper000 {
    fn write_to_cartridge_space(&mut self, address: CpuAddress, _value: u8) {
        match address.to_raw() {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0xFFFF => { /* Only mapper 0 does nothing here. */ },
        }
    }

    fn params(&self) -> &MapperParams { &self.params }
    fn params_mut(&mut self) -> &mut MapperParams { &mut self.params }
}

impl Mapper000 {
    pub fn new(cartridge: &Cartridge) -> Result<Mapper000, String> {
        Ok(Mapper000 {
            params: INITIAL_LAYOUT.make_mapper_params(cartridge),
        })
    }
}
