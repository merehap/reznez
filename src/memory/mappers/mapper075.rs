use crate::memory::mapper::*;

const PRG_WINDOWS: PrgWindows = PrgWindows::new(&[
    PrgWindow::new(0x6000, 0x7FFF, 8 * KIBIBYTE, PrgType::Empty),
    PrgWindow::new(0x8000, 0x9FFF, 8 * KIBIBYTE, PrgType::SwitchableBank(Rom, P0)),
    PrgWindow::new(0xA000, 0xBFFF, 8 * KIBIBYTE, PrgType::SwitchableBank(Rom, P1)),
    PrgWindow::new(0xC000, 0xDFFF, 8 * KIBIBYTE, PrgType::SwitchableBank(Rom, P2)),
    PrgWindow::new(0xE000, 0xFFFF, 8 * KIBIBYTE, PrgType::FixedBank(Rom, BankIndex::LAST)),
]);

const CHR_WINDOWS: ChrWindows = ChrWindows::new(&[
    ChrWindow::new(0x0000, 0x0FFF, 4 * KIBIBYTE, ChrType::SwitchableBank(Rom, C0)),
    ChrWindow::new(0x1000, 0x1FFF, 4 * KIBIBYTE, ChrType::SwitchableBank(Rom, C1)),
]);

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
            .prg_windows(PRG_WINDOWS)
            .chr_max_bank_count(32)
            .chr_bank_size(4 * KIBIBYTE)
            .chr_windows(CHR_WINDOWS)
            .name_table_mirroring_source(NameTableMirroringSource::Cartridge)
            .build()
    }

    fn write_to_cartridge_space(&mut self, params: &mut MapperParams, cpu_address: CpuAddress, value: u8) {
        match cpu_address.to_raw() {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x7FFF => { /* Do nothing. */ }
            0x8000..=0x8FFF =>
                params.prg_memory_mut().set_bank_index_register(P0, value & 0b0000_1111),
            0xA000..=0xAFFF =>
                params.prg_memory_mut().set_bank_index_register(P1, value & 0b0000_1111),
            0xC000..=0xCFFF =>
                params.prg_memory_mut().set_bank_index_register(P2, value & 0b0000_1111),
            0x9000..=0x9FFF => {
                if params.name_table_mirroring() != NameTableMirroring::FourScreen {
                    let mirroring = if value & 0b001 == 0 {
                        NameTableMirroring::Vertical
                    } else {
                        NameTableMirroring::Horizontal
                    };
                    params.set_name_table_mirroring(mirroring);
                }

                self.chr_left_high_bit = (value & 0b010) << 3;
                self.chr_right_high_bit = (value & 0b100) << 2;
            }
            0xE000..=0xEFFF => {
                let bank_index = self.chr_left_high_bit | (value & 0b0000_1111);
                params.chr_memory_mut().set_bank_index_register(C0, bank_index);
            }
            0xF000..=0xFFFF => {
                let bank_index = self.chr_right_high_bit | (value & 0b0000_1111);
                params.chr_memory_mut().set_bank_index_register(C1, bank_index);
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