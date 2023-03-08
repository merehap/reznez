use crate::memory::mapper::*;

const INITIAL_LAYOUT: InitialLayout = InitialLayout::builder()
    .prg_max_bank_count(16)
    .prg_bank_size(16 * KIBIBYTE)
    .prg_windows_by_board(&[(Board::Any, PRG_WINDOWS)])
    .chr_max_bank_count(1)
    .chr_bank_size(8 * KIBIBYTE)
    .chr_windows(CHR_WINDOWS)
    .name_table_mirroring_source(NameTableMirroringSource::Cartridge)
    .build();

const PRG_WINDOWS: &[PrgWindow] = &[
    PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgType::Empty),
    PrgWindow::new(0x8000, 0xBFFF, 16 * KIBIBYTE, PrgType::Banked(Rom, BankIndex::Register(P0))),
    PrgWindow::new(0xC000, 0xFFFF, 16 * KIBIBYTE, PrgType::Banked(Rom, BankIndex::LAST)),
];

const CHR_WINDOWS: &[ChrWindow] = &[
    ChrWindow::new(0x0000, 0x1FFF, 8 * KIBIBYTE, ChrType(Rom, BankIndex::FIRST)),
];

// Similar to UxROM.
pub struct Mapper071 {
    params: MapperParams,
}

impl Mapper for Mapper071 {
    fn write_to_cartridge_space(&mut self, address: CpuAddress, value: u8) {
        let bank_index = value & 0b1111;
        match address.to_raw() {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x8FFF => { /* Do nothing. */ }
            // https://www.nesdev.org/wiki/INES_Mapper_071#Mirroring_($8000-$9FFF)
            0x9000..=0x9FFF => {
                let mirroring = if value & 0b0001_0000 == 0 {
                    NameTableMirroring::OneScreenLeftBank
                } else {
                    NameTableMirroring::OneScreenRightBank
                };
                self.set_name_table_mirroring(mirroring);
            }
            0xA000..=0xBFFF => { /* Do nothing. */ }
            0xC000..=0xFFFF => self.prg_memory_mut().set_bank_index_register(P0, bank_index),
        }
    }

    fn params(&self) -> &MapperParams { &self.params }
    fn params_mut(&mut self) -> &mut MapperParams { &mut self.params }
}

impl Mapper071 {
    pub fn new(cartridge: &Cartridge) -> Result<Mapper071, String> {
        Ok(Mapper071 {
            params: INITIAL_LAYOUT.make_mapper_params(cartridge, Board::Any),
        })
    }
}
