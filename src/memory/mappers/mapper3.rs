use crate::cartridge::Cartridge;
use crate::memory::cpu::cpu_address::CpuAddress;
use crate::memory::ppu::chr_memory::{ChrMemory, ChrType, AddDefaultRamIfRomMissing};
use crate::memory::cpu::prg_memory::{PrgMemory, PrgType};
use crate::memory::mapper::*;
use crate::ppu::name_table::name_table_mirroring::NameTableMirroring;
use crate::util::unit::KIBIBYTE;

// CNROM
pub struct Mapper3 {
    prg_memory: PrgMemory,
    chr_memory: ChrMemory,
    name_table_mirroring: NameTableMirroring,
}

impl Mapper3 {
    pub fn new(cartridge: &Cartridge) -> Result<Mapper3, String> {
        validate_chr_data_length(cartridge, |len| len > 0 && len <= 2048 * KIBIBYTE)?;

        // Same as Mapper 0.
        let prg_rom_len = cartridge.prg_rom().len();
        let prg_memory = if prg_rom_len == 16 * KIBIBYTE {
            PrgMemory::builder()
                .raw_memory(cartridge.prg_rom())
                .bank_count(1)
                .bank_size(16 * KIBIBYTE)
                .add_window(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgType::Empty)
                .add_window(0x8000, 0xBFFF, 16 * KIBIBYTE, PrgType::Rom { bank_index: 0 })
                .add_window(0xC000, 0xFFFF, 16 * KIBIBYTE, PrgType::MirrorPrevious)
                .build()
        } else if prg_rom_len == 32 * KIBIBYTE {
            PrgMemory::builder()
                .raw_memory(cartridge.prg_rom())
                .bank_count(1)
                .bank_size(32 * KIBIBYTE)
                .add_window(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgType::Empty)
                .add_window(0x8000, 0xFFFF, 32 * KIBIBYTE, PrgType::Rom { bank_index: 0 })
                .build()
        } else {
            return Err("PRG ROM size must be 16K or 32K for mapper 3.".to_string());
        };

        let chr_memory = ChrMemory::builder()
            .raw_memory(cartridge.chr_rom())
            .bank_count((cartridge.chr_rom().len() / (8 * KIBIBYTE)).try_into().unwrap())
            .bank_size(8 * KIBIBYTE)
            .add_window(0x0000, 0x1FFF, 8 * KIBIBYTE, ChrType::Rom { bank_index: 0 })
            .build(AddDefaultRamIfRomMissing::No);

        Ok(Mapper3 {
            prg_memory,
            chr_memory,
            name_table_mirroring: cartridge.name_table_mirroring(),
        })
    }
}

impl Mapper for Mapper3 {
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

    fn write_to_prg_memory(&mut self, cpu_address: CpuAddress, value: u8) {
        match cpu_address.to_raw() {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x7FFF => { /* Do nothing. */ },
            0x8000..=0xFFFF => self.chr_memory.switch_bank_at(0x0000, value),
        }
    }
}
