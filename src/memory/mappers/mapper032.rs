use crate::memory::mapper::*;

const INITIAL_LAYOUT: InitialLayout = InitialLayout::builder()
    .prg_max_bank_count(32)
    .prg_bank_size(8 * KIBIBYTE)
    .prg_windows_by_board(&[(Board::Any, PRG_WINDOWS_LAST_TWO_FIXED)])
    .chr_max_bank_count(256)
    .chr_bank_size(1 * KIBIBYTE)
    .chr_windows(CHR_WINDOWS)
    .name_table_mirroring_source(NameTableMirroringSource::Cartridge)
    .build();

const PRG_WINDOWS_LAST_TWO_FIXED: PrgWindows = PrgWindows::new(&[
    PrgWindow::new(0x6000, 0x7FFF, 8 * KIBIBYTE, PrgType::Empty),
    PrgWindow::new(0x8000, 0x9FFF, 8 * KIBIBYTE, PrgType::VariableBank(Rom, P0)),
    PrgWindow::new(0xA000, 0xBFFF, 8 * KIBIBYTE, PrgType::VariableBank(Rom, P1)),
    PrgWindow::new(0xC000, 0xDFFF, 8 * KIBIBYTE, PrgType::ConstantBank(Rom, BankIndex::SECOND_LAST)),
    PrgWindow::new(0xE000, 0xFFFF, 8 * KIBIBYTE, PrgType::ConstantBank(Rom, BankIndex::LAST)),
]);
const PRG_WINDOWS_ENDS_FIXED: PrgWindows = PrgWindows::new(&[
    PrgWindow::new(0x6000, 0x7FFF, 8 * KIBIBYTE, PrgType::Empty),
    PrgWindow::new(0x8000, 0x9FFF, 8 * KIBIBYTE, PrgType::ConstantBank(Rom, BankIndex::SECOND_LAST)),
    PrgWindow::new(0xA000, 0xBFFF, 8 * KIBIBYTE, PrgType::VariableBank(Rom, P1)),
    PrgWindow::new(0xC000, 0xDFFF, 8 * KIBIBYTE, PrgType::VariableBank(Rom, P0)),
    PrgWindow::new(0xE000, 0xFFFF, 8 * KIBIBYTE, PrgType::ConstantBank(Rom, BankIndex::LAST)),
]);

const CHR_WINDOWS: ChrWindows = ChrWindows::new(&[
    ChrWindow::new(0x0000, 0x03FF, 1 * KIBIBYTE, ChrType::VariableBank(Rom, C0)),
    ChrWindow::new(0x0400, 0x07FF, 1 * KIBIBYTE, ChrType::VariableBank(Rom, C1)),
    ChrWindow::new(0x0800, 0x0BFF, 1 * KIBIBYTE, ChrType::VariableBank(Rom, C2)),
    ChrWindow::new(0x0C00, 0x0FFF, 1 * KIBIBYTE, ChrType::VariableBank(Rom, C3)),
    ChrWindow::new(0x1000, 0x13FF, 1 * KIBIBYTE, ChrType::VariableBank(Rom, C4)),
    ChrWindow::new(0x1400, 0x17FF, 1 * KIBIBYTE, ChrType::VariableBank(Rom, C5)),
    ChrWindow::new(0x1800, 0x1BFF, 1 * KIBIBYTE, ChrType::VariableBank(Rom, C6)),
    ChrWindow::new(0x1C00, 0x1FFF, 1 * KIBIBYTE, ChrType::VariableBank(Rom, C7)),
]);

// Irem's G-101
pub struct Mapper032 {
    params: MapperParams,
}

impl Mapper for Mapper032 {
    fn write_to_cartridge_space(&mut self, address: CpuAddress, value: u8) {
        match address.to_raw() {
            0x0000..=0x401F => unreachable!(),
            0x8000..=0x8007 => self.prg_memory_mut().set_bank_index_register(P0, value & 0b1_1111),
            0x9000..=0x9007 => {
                let windows = if value & 0b10 == 0 {
                    PRG_WINDOWS_LAST_TWO_FIXED
                } else {
                    PRG_WINDOWS_ENDS_FIXED
                };
                self.prg_memory_mut().set_windows(windows);

                let mirroring = if value & 0b01 == 0 {
                    NameTableMirroring::Vertical
                } else {
                    NameTableMirroring::Horizontal
                };
                self.set_name_table_mirroring(mirroring);
            }
            0xA000..=0xA007 => self.prg_memory_mut().set_bank_index_register(P1, value & 0b1_1111),
            0xB000 => self.chr_memory_mut().set_bank_index_register(C0, value),
            0xB001 => self.chr_memory_mut().set_bank_index_register(C1, value),
            0xB002 => self.chr_memory_mut().set_bank_index_register(C2, value),
            0xB003 => self.chr_memory_mut().set_bank_index_register(C3, value),
            0xB004 => self.chr_memory_mut().set_bank_index_register(C4, value),
            0xB005 => self.chr_memory_mut().set_bank_index_register(C5, value),
            0xB006 => self.chr_memory_mut().set_bank_index_register(C6, value),
            0xB007 => self.chr_memory_mut().set_bank_index_register(C7, value),
            _ => { /* Do nothing. */ }
        }
    }

    fn params(&self) -> &MapperParams { &self.params }
    fn params_mut(&mut self) -> &mut MapperParams { &mut self.params }
}

impl Mapper032 {
    pub fn new(cartridge: &Cartridge) -> Result<Mapper032, String> {
        Ok(Mapper032 {
            params: INITIAL_LAYOUT.make_mapper_params(cartridge, Board::Any),
        })
    }
}
