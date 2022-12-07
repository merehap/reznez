use crate::cartridge::Cartridge;
use crate::memory::cpu::cpu_address::CpuAddress;
use crate::memory::ppu::chr_memory::{ChrMemory, ChrType, AddDefaultRamIfRomMissing};
use crate::memory::cpu::prg_memory::{PrgMemory, PrgType};
use crate::memory::mapper::*;
use crate::ppu::name_table::name_table_mirroring::NameTableMirroring;
use crate::util::unit::KIBIBYTE;

const BANK_SELECT_START: CpuAddress = CpuAddress::new(0x8000);

// CNROM
pub struct Mapper3 {
    prg_memory: PrgMemory,
    chr_memory: ChrMemory,
    name_table_mirroring: NameTableMirroring,
}

impl Mapper3 {
    pub fn new(cartridge: &Cartridge) -> Result<Mapper3, String> {
        let prg_rom_chunks = cartridge.prg_rom_chunks();
        let prg_memory = match prg_rom_chunks.len() {
            1 => PrgMemory::builder()
                    .raw_memory(prg_rom_chunks[0].to_vec())
                    .bank_count(1)
                    .bank_size(16 * KIBIBYTE)
                    .add_window(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgType::Empty)
                    .add_window(0x8000, 0xBFFF, 16 * KIBIBYTE, PrgType::Rom { bank_index: 0 })
                    .add_window(0xC000, 0xFFFF, 16 * KIBIBYTE, PrgType::MirrorPrevious)
                    .build(),
            2 => PrgMemory::builder()
                    .raw_memory(cartridge.prg_rom())
                    .bank_count(1)
                    .bank_size(32 * KIBIBYTE)
                    .add_window(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgType::Empty)
                    .add_window(0x8000, 0xFFFF, 32 * KIBIBYTE, PrgType::Rom { bank_index: 0 })
                    .build(),
            c => {
                return Err(format!(
                    "PRG ROM size must be 16K or 32K for this mapper, but was {}K",
                    16 * c,
                ))
            }
        };

        let chr_chunk_count = cartridge.chr_rom_chunks().len();
        if chr_chunk_count > 256 {
            return Err(format!(
                "Max CHR chunks for Mapper 3 is 256, but found {}.",
                chr_chunk_count,
            ));
        }

        let chr_memory = ChrMemory::builder()
            .raw_memory(cartridge.chr_rom())
            .bank_count(chr_chunk_count.try_into().unwrap())
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
        if cpu_address >= BANK_SELECT_START {
            self.chr_memory.switch_bank_at(0x0000, value);
        }
    }
}
