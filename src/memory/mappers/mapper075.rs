use crate::memory::mapper::*;

const PRG_LAYOUT: PrgLayout = PrgLayout::new(&[
    PrgWindow::new(0x6000, 0x7FFF, 8 * KIBIBYTE, Bank::EMPTY),
    PrgWindow::new(0x8000, 0x9FFF, 8 * KIBIBYTE, Bank::switchable_rom(P0)),
    PrgWindow::new(0xA000, 0xBFFF, 8 * KIBIBYTE, Bank::switchable_rom(P1)),
    PrgWindow::new(0xC000, 0xDFFF, 8 * KIBIBYTE, Bank::switchable_rom(P2)),
    PrgWindow::new(0xE000, 0xFFFF, 8 * KIBIBYTE, Bank::fixed_rom(BankIndex::LAST)),
]);

const CHR_LAYOUT: ChrLayout = ChrLayout::new(&[
    ChrWindow::new(0x0000, 0x0FFF, 4 * KIBIBYTE, ChrBank::Switchable(Rom, C0)),
    ChrWindow::new(0x1000, 0x1FFF, 4 * KIBIBYTE, ChrBank::Switchable(Rom, C1)),
]);

const MIRRORINGS: [NameTableMirroring; 2] = [
    NameTableMirroring::Vertical,
    NameTableMirroring::Horizontal,
];

// VRC1
pub struct Mapper075 {
    chr_left_high_bit: u8,
    chr_right_high_bit: u8,
}

impl Mapper for Mapper075 {
    fn initial_layout(&self) -> InitialLayout {
        InitialLayout::builder()
            .prg_max_bank_count(16)
            .prg_bank_size(8 * KIBIBYTE)
            .prg_windows(PRG_LAYOUT)
            .chr_max_bank_count(32)
            .chr_bank_size(4 * KIBIBYTE)
            .chr_windows(CHR_LAYOUT)
            .name_table_mirroring_source(NameTableMirroringSource::Cartridge)
            .build()
    }

    fn write_to_cartridge_space(&mut self, params: &mut MapperParams, cpu_address: CpuAddress, value: u8) {
        match cpu_address.to_raw() {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x7FFF => { /* Do nothing. */ }
            0x8000..=0x8FFF =>
                params.set_bank_register(P0, value & 0b0000_1111),
            0xA000..=0xAFFF =>
                params.set_bank_register(P1, value & 0b0000_1111),
            0xC000..=0xCFFF =>
                params.set_bank_register(P2, value & 0b0000_1111),
            0x9000..=0x9FFF => {
                if params.name_table_mirroring() != NameTableMirroring::FourScreen {
                    params.set_name_table_mirroring(MIRRORINGS[usize::from(value & 0b001)]);
                }

                self.chr_left_high_bit = (value & 0b010) << 3;
                self.chr_right_high_bit = (value & 0b100) << 2;
            }
            0xE000..=0xEFFF => {
                let bank_index = self.chr_left_high_bit | (value & 0b0000_1111);
                params.set_bank_register(C0, bank_index);
            }
            0xF000..=0xFFFF => {
                let bank_index = self.chr_right_high_bit | (value & 0b0000_1111);
                params.set_bank_register(C1, bank_index);
            }
            0xB000..=0xBFFF | 0xD000..=0xDFFF => { /* No registers here. */ }
        }
    }
}

impl Mapper075 {
    pub fn new() -> Self {
        Self {
            chr_left_high_bit: 0b0000_0000,
            chr_right_high_bit: 0b0000_0000,
        }
    }
}
