use crate::memory::mapper::*;

const INITIAL_LAYOUT: InitialLayout = InitialLayout::builder()
    // TODO: Figure out how to fix this for mirrored memory, if necessary
    .prg_max_bank_count(2)
    .prg_bank_size(16 * KIBIBYTE)
    .prg_windows_by_board(&[
        (Board::Cnrom128, PRG_WINDOWS_CNROM_128),
        (Board::Cnrom256, PRG_WINDOWS_CNROM_256),
    ])
    .chr_max_bank_count(256)
    .chr_bank_size(8 * KIBIBYTE)
    .chr_windows(CHR_WINDOWS)
    .name_table_mirroring_source(NameTableMirroringSource::Cartridge)
    .build();

const PRG_WINDOWS_CNROM_128: &[PrgWindow] = &[
    PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgType::Empty),
    PrgWindow::new(0x8000, 0xBFFF, 16 * KIBIBYTE, PrgType::ConstantBank(Rom, BankIndex::FIRST)),
    PrgWindow::new(0xC000, 0xFFFF, 16 * KIBIBYTE, PrgType::Mirror(0x8000)),
];
const PRG_WINDOWS_CNROM_256: &[PrgWindow] = &[
    PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgType::Empty),
    PrgWindow::new(0x8000, 0xFFFF, 32 * KIBIBYTE, PrgType::ConstantBank(Rom, BankIndex::FIRST)),
];

const CHR_WINDOWS: &[ChrWindow] = &[
    ChrWindow::new(0x0000, 0x1FFF, 8 * KIBIBYTE, ChrType::VariableBank(Rom, C0)),
];

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
        let prg_board = Mapper003::board(cartridge)?;
        Ok(Mapper003 {
            params: INITIAL_LAYOUT.make_mapper_params(cartridge, prg_board),
        })
    }

    fn board(cartridge: &Cartridge) -> Result<Board, String> {
        let prg_rom_len = cartridge.prg_rom().len();
        if prg_rom_len == 16 * KIBIBYTE {
            Ok(Board::Cnrom128)
        } else if prg_rom_len == 32 * KIBIBYTE {
            Ok(Board::Cnrom256)
        } else {
            Err("PRG ROM size must be 16K or 32K for mapper 0.".to_string())
        }
    }
}
