use crate::memory::mapper::*;

const INITIAL_LAYOUT: InitialLayout = InitialLayout {
    // TODO: Figure out how to fix this for mirrored memory, if necessary
    prg_max_bank_count: 256,
    prg_bank_size: 16 * KIBIBYTE,
    prg_windows_by_board: &[(Board::Any, PRG_WINDOWS)],

    chr_max_bank_count: 1,
    chr_bank_size: 8 * KIBIBYTE,
    chr_windows: CHR_WINDOWS,

    name_table_mirroring_source: NameTableMirroringSource::Cartridge,
};

const PRG_WINDOWS: &[PrgWindow] = &[
    PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgType::Empty),
    PrgWindow::new(0x8000, 0xBFFF, 16 * KIBIBYTE, PrgType::Banked(Rom, BankIndex::Register(P0))),
    PrgWindow::new(0xC000, 0xFFFF, 16 * KIBIBYTE, PrgType::Banked(Rom, BankIndex::LAST)),
];

const CHR_WINDOWS: &[ChrWindow] = &[
    ChrWindow::new(0x0000, 0x1FFF, 8 * KIBIBYTE, ChrType(Rom, BankIndex::FIRST)),
];

// UxROM (common usages)
pub struct Mapper2 {
    params: MapperParams,
}

impl Mapper for Mapper2 {
    fn write_to_cartridge_space(&mut self, address: CpuAddress, value: u8) {
        match address.to_raw() {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x7FFF => { /* Do nothing. */ },
            0x8000..=0xFFFF => self.prg_memory_mut().set_bank_index_register(P0, value),
        }
    }

    fn params(&self) -> &MapperParams { &self.params }
    fn params_mut(&mut self) -> &mut MapperParams { &mut self.params }
}

impl Mapper2 {
    pub fn new(cartridge: &Cartridge) -> Result<Mapper2, String> {
        let params = INITIAL_LAYOUT.make_mapper_params(cartridge, Board::Any);
        Ok(Mapper2 { params })
    }
}
