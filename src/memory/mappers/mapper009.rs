use crate::memory::mapper::*;

const INITIAL_LAYOUT: InitialLayout = InitialLayout::builder()
    .prg_max_bank_count(32)
    .prg_bank_size(8 * KIBIBYTE)
    .prg_windows_by_board(&[(Board::Any, PRG_WINDOWS)])
    .chr_max_bank_count(256)
    .chr_bank_size(4 * KIBIBYTE)
    .chr_windows(CHR_WINDOWS)
    .name_table_mirroring_source(NameTableMirroringSource::Cartridge)
    .build();

const PRG_WINDOWS: PrgWindows = PrgWindows::new(&[
    // TODO: PlayChoice uses this window.
    PrgWindow::new(0x6000, 0x7FFF, 8 * KIBIBYTE, PrgType::Empty),
    PrgWindow::new(0x8000, 0x9FFF, 8 * KIBIBYTE, PrgType::VariableBank(Rom, P0)),
    PrgWindow::new(0xA000, 0xBFFF, 8 * KIBIBYTE, PrgType::ConstantBank(Rom, BankIndex::THIRD_LAST)),
    PrgWindow::new(0xC000, 0xDFFF, 8 * KIBIBYTE, PrgType::ConstantBank(Rom, BankIndex::SECOND_LAST)),
    PrgWindow::new(0xE000, 0xFFFF, 8 * KIBIBYTE, PrgType::ConstantBank(Rom, BankIndex::LAST)),
]);

const CHR_WINDOWS: ChrWindows = ChrWindows::new(&[
    ChrWindow::new(0x0000, 0x0FFF, 4 * KIBIBYTE, ChrType::MetaVariableBank(Rom, M0)),
    ChrWindow::new(0x1000, 0x1FFF, 4 * KIBIBYTE, ChrType::MetaVariableBank(Rom, M1)),
]);

// MMC2
pub struct Mapper009 {
    params: MapperParams,
}

impl Mapper for Mapper009 {
    fn write_to_cartridge_space(&mut self, address: CpuAddress, value: u8) {
        let bank_index = value & 0b0001_1111;
        match address.to_raw() {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x9FFF => { /* Do nothing. */ }
            0xA000..=0xAFFF => self.prg_memory_mut().set_bank_index_register(P0, bank_index),
            0xB000..=0xBFFF => self.chr_memory_mut().set_bank_index_register(C0, bank_index),
            0xC000..=0xCFFF => self.chr_memory_mut().set_bank_index_register(C1, bank_index),
            0xD000..=0xDFFF => self.chr_memory_mut().set_bank_index_register(C2, bank_index),
            0xE000..=0xEFFF => self.chr_memory_mut().set_bank_index_register(C3, bank_index),
            0xF000..=0xFFFF => {
                let mirroring = if value & 1 == 0 {
                    NameTableMirroring::Vertical
                } else {
                    NameTableMirroring::Horizontal
                };
                self.set_name_table_mirroring(mirroring);
            }
        }
    }

    fn on_ppu_read(&mut self, address: PpuAddress, _value: u8) {
        let (meta_id, bank_index_register_id) = match address.to_u16() {
            0x0FD8 => (M0, C0),
            0x0FE8 => (M0, C1),
            0x1FD8..=0x1FDF => (M1, C2),
            0x1FE8..=0x1FEF => (M1, C3),
            // Skip to standard CHR memory operation.
            _ => return,
        };

        self.chr_memory_mut().set_meta_register(meta_id, bank_index_register_id);
    }

    fn params(&self) -> &MapperParams { &self.params }
    fn params_mut(&mut self) -> &mut MapperParams { &mut self.params }
}

impl Mapper009 {
    pub fn new(cartridge: &Cartridge) -> Result<Mapper009, String> {
        Ok(Mapper009 {
            params: INITIAL_LAYOUT.make_mapper_params(cartridge, Board::Any),
        })
    }
}