use crate::memory::mapper::*;

// NROM
pub struct Mapper0 {
    prg_memory: PrgMemory,
    chr_memory: ChrMemory,
    name_table_mirroring: NameTableMirroring,
}

impl Mapper0 {
    pub fn new(cartridge: &Cartridge) -> Result<Mapper0, String> {
        // Not bank-switched.
        let prg_memory = match Mapper0::board(cartridge)? {
            Board::Nrom128 => PrgMemory::builder()
                .raw_memory(cartridge.prg_rom())
                .max_bank_count(1)
                .bank_size(16 * KIBIBYTE)
                .add_window(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgType::Empty)
                .add_window(0x8000, 0xBFFF, 16 * KIBIBYTE, PrgType::Rom { bank_index: 0 })
                .add_window(0xC000, 0xFFFF, 16 * KIBIBYTE, PrgType::MirrorPrevious)
                .build(),
            Board::Nrom256 => PrgMemory::builder()
                .raw_memory(cartridge.prg_rom())
                .max_bank_count(1)
                .bank_size(32 * KIBIBYTE)
                .add_window(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgType::Empty)
                .add_window(0x8000, 0xFFFF, 32 * KIBIBYTE, PrgType::Rom { bank_index: 0 })
                .build(),
        };

        // Not bank-switched.
        let chr_memory = ChrMemory::builder()
            .raw_memory(cartridge.chr_rom())
            .max_bank_count(1)
            .bank_size(8 * KIBIBYTE)
            .add_window(0x0000, 0x1FFF, 8 * KIBIBYTE, ChrType::Rom { bank_index: 0 })
            .add_default_ram_if_chr_data_missing();

        Ok(Mapper0 {
            prg_memory,
            chr_memory,
            name_table_mirroring: cartridge.name_table_mirroring(),
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

impl Mapper for Mapper0 {
    fn write_to_cartridge_space(&mut self, address: CpuAddress, _value: u8) {
        match address.to_raw() {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0xFFFF => { /* Only mapper 0 does nothing here. */ },
        }
    }

    fn name_table_mirroring(&self) -> NameTableMirroring {
        self.name_table_mirroring
    }

    fn prg_memory(&self) -> &PrgMemory {
        &self.prg_memory
    }

    fn chr_memory(&self) -> &ChrMemory {
        &self.chr_memory
    }

    fn chr_memory_mut(&mut self) -> &mut ChrMemory {
        &mut self.chr_memory
    }
}

enum Board {
    Nrom128,
    Nrom256,
}
