use crate::memory::mapper::*;

const INITIAL_LAYOUT: InitialLayout = InitialLayout::builder()
    .prg_max_bank_count(1)
    .prg_bank_size(32 * KIBIBYTE)
    .prg_windows(PRG_WINDOWS)
    .chr_max_bank_count(256)
    .chr_bank_size(8 * KIBIBYTE)
    .chr_windows(CHR_WINDOWS)
    .name_table_mirroring_source(NameTableMirroringSource::Cartridge)
    .build();

const PRG_WINDOWS: PrgWindows = PrgWindows::new(&[
    PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgType::Empty),
    PrgWindow::new(0x8000, 0xFFFF, 32 * KIBIBYTE, PrgType::ConstantBank(Rom, BankIndex::FIRST)),
]);

const CHR_WINDOWS: ChrWindows = ChrWindows::new(&[
    ChrWindow::new(0x0000, 0x1FFF, 8 * KIBIBYTE, ChrType::VariableBank(Rom, C0)),
]);

// CNROM
pub struct Mapper003 {
    params: MapperParams,
}

impl Mapper for Mapper003 {
    fn write_to_cartridge_space(&mut self, cpu_address: CpuAddress, value: u8) {
        match cpu_address.to_raw() {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x7FFF => { /* Do nothing. */ },
            0x8000..=0xFFFF => self.chr_memory_mut().set_bank_index_register(C0, value),
        }
    }

    fn params(&self) -> &MapperParams { &self.params }
    fn params_mut(&mut self) -> &mut MapperParams { &mut self.params }
}

impl Mapper003 {
    pub fn new(cartridge: &Cartridge) -> Result<Mapper003, String> {
        Ok(Mapper003 {
            params: INITIAL_LAYOUT.make_mapper_params(cartridge),
        })
    }
}
