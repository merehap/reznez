use crate::memory::mapper::*;

const PRG_LAYOUT: PrgLayout = PrgLayout::new(&[
    PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgType::Empty),
    PrgWindow::new(0x8000, 0xFFFF, 32 * KIBIBYTE, PrgType::SwitchableBank(Rom, P0)),
]);

const CHR_LAYOUT: ChrLayout = ChrLayout::new(&[
    ChrWindow::new(0x0000, 0x0FFF, 4 * KIBIBYTE, ChrType::SwitchableBank(Ram, C0)),
    ChrWindow::new(0x1000, 0x1FFF, 4 * KIBIBYTE, ChrType::SwitchableBank(Ram, C1)),
]);

// BNROM (BxROM) and NINA-01. Two unrelated mappers combined into one.
pub struct Mapper034 {
    board: MapperBoard,
}

impl Mapper for Mapper034 {
    fn initial_layout(&self) -> InitialLayout {
        InitialLayout::builder()
            .prg_max_bank_count(256)
            .prg_bank_size(32 * KIBIBYTE)
            .prg_windows(PRG_LAYOUT)
            .chr_max_bank_count(16)
            .chr_bank_size(4 * KIBIBYTE)
            .chr_windows(CHR_LAYOUT)
            .name_table_mirroring_source(NameTableMirroringSource::Cartridge)
            .override_bank_index_register(C1, BankIndex::LAST)
            .build()
    }

    fn write_to_cartridge_space(&mut self, params: &mut MapperParams, address: CpuAddress, value: u8) {
        match address.to_raw() {
            0x0000..=0x401F => unreachable!(),
            // NINA-001 bank-switching.
            0x7FFD => params.prg_memory_mut().set_bank_index_register(P0, value & 1),
            0x7FFE => params.chr_memory_mut().set_bank_index_register(C0, value & 0b1111),
            0x7FFF => params.chr_memory_mut().set_bank_index_register(C1, value & 0b1111),
            // BNROM/BxROM bank-switching.
            0x8000..=0xFFFF => {
                if self.board == MapperBoard::BxROM {
                    params.prg_memory_mut().set_bank_index_register(P0, value);
                }
            }
            _ => { /* Do nothing. */ }
        }
    }
}

impl Mapper034 {
    pub fn new(cartridge: &Cartridge) -> Self {
        let board = if cartridge.chr_rom().len() <= 8 * KIBIBYTE {
            MapperBoard::BxROM
        } else {
            MapperBoard::Nina001
        };

        Self { board }
    }
}

#[derive(PartialEq, Eq)]
enum MapperBoard {
    BxROM,
    Nina001,
}
