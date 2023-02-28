use crate::memory::mapper::*;

const INITIAL_LAYOUT: InitialLayout = InitialLayout {
    prg_max_bank_count: 4,
    prg_bank_size: 32 * KIBIBYTE,
    prg_windows_by_board: &[(Board::Any, PRG_WINDOWS)],

    chr_max_bank_count: 4,
    chr_bank_size: 8 * KIBIBYTE,
    chr_windows: CHR_WINDOWS,

    name_table_mirroring_source: NameTableMirroringSource::Cartridge,
};

const PRG_WINDOWS: &[PrgWindow] = &[
    PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgType::Empty),
    PrgWindow::new(0x8000, 0xFFFF, 32 * KIBIBYTE, PrgType::Banked(Rom, BankIndex::Register(P0))),
];

const CHR_WINDOWS: &[ChrWindow] = &[
    ChrWindow::new(0x0000, 0x1FFF, 8 * KIBIBYTE, ChrType(Rom, BankIndex::Register(C0))),
];

// GxROM
pub struct Mapper066 {
    params: MapperParams,
}

impl Mapper for Mapper066 {
    fn write_to_cartridge_space(&mut self, cpu_address: CpuAddress, value: u8) {
        match cpu_address.to_raw() {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x7FFF => { /* Do nothing. */ },
            0x8000..=0xFFFF => {
                assert_eq!(value & 0b1100_1100, 0);
                self.prg_memory_mut().set_bank_index_register(P0, (value & 0b0011_0000) >> 4);
                self.chr_memory_mut().set_bank_index_register(C0, value & 0b0000_0011);
            }
        }
    }

    fn params(&self) -> &MapperParams { &self.params }
    fn params_mut(&mut self) -> &mut MapperParams { &mut self.params }
}

impl Mapper066 {
    pub fn new(cartridge: &Cartridge) -> Result<Mapper066, String> {
        Ok(Mapper066 {
            params: INITIAL_LAYOUT.make_mapper_params(cartridge, Board::Any),
        })
    }
}
