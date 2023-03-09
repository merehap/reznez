use crate::memory::mapper::*;

const INITIAL_LAYOUT: InitialLayout = InitialLayout::builder()
    .prg_max_bank_count(2)
    .prg_bank_size(16 * KIBIBYTE)
    .prg_windows_by_board(&[
        (Board::Nrom128, PRG_WINDOWS_NROM_128),
        (Board::Nrom256, PRG_WINDOWS_NROM_256),
    ])
    .chr_max_bank_count(1)
    .chr_bank_size(8 * KIBIBYTE)
    .chr_windows(CHR_WINDOWS)
    .name_table_mirroring_source(NameTableMirroringSource::Cartridge)
    .build();

const PRG_WINDOWS_NROM_128: PrgWindows = PrgWindows::new(&[
    PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgType::Empty),
    PrgWindow::new(0x8000, 0xBFFF, 16 * KIBIBYTE, PrgType::ConstantBank(Rom, BankIndex::FIRST)),
    PrgWindow::new(0xC000, 0xFFFF, 16 * KIBIBYTE, PrgType::Mirror(0x8000)),
]);
const PRG_WINDOWS_NROM_256: PrgWindows = PrgWindows::new(&[
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
        let board = Mapper000::board(cartridge)?;
        Ok(Mapper000 {
            params: INITIAL_LAYOUT.make_mapper_params(cartridge, board),
        })
    }

    fn board(cartridge: &Cartridge) -> Result<Board, String> {
        let prg_rom_len = cartridge.prg_rom().len();
        if prg_rom_len == 16 * KIBIBYTE {
            Ok(Board::Nrom128)
        } else if prg_rom_len == 32 * KIBIBYTE {
            Ok(Board::Nrom256)
        } else {
            Err("PRG ROM size must be 16K or 32K for mapper 0.".to_string())
        }
    }
}
