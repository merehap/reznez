use crate::cartridge::Cartridge;
use crate::memory::cpu::cpu_address::CpuAddress;
use crate::memory::ppu::chr_memory::{ChrMemory, ChrType};
use crate::memory::cpu::prg_memory::{PrgMemory, PrgType};
use crate::memory::mapper::*;
use crate::ppu::name_table::name_table_mirroring::NameTableMirroring;
use crate::ppu::pattern_table::PatternTableSide;
use crate::util::mapped_array::{Chunk, MappedArray};
use crate::util::unit::KIBIBYTE;

// NROM
pub struct Mapper0 {
    prg_memory: PrgMemory,
    raw_pattern_tables: RawPatternTablePair,
    name_table_mirroring: NameTableMirroring,
    chr_memory: ChrMemory,
    is_chr_writable: bool,
}

impl Mapper0 {
    pub fn new(cartridge: &Cartridge) -> Result<Mapper0, String> {
        let prg_rom_chunks = cartridge.prg_rom_chunks();
        let prg_memory = match prg_rom_chunks.len() {
            /* Nrom128 - Mirrored mappings. */
            1 => PrgMemory::builder()
                    .raw_memory(cartridge.prg_rom_chunks()[0].to_vec())
                    .bank_count(1)
                    .bank_size(16 * KIBIBYTE)
                    .add_window(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgType::Empty)
                    .add_window(0x8000, 0xBFFF, 16 * KIBIBYTE, PrgType::Rom { bank_index: 0 })
                    .add_window(0xC000, 0xFFFF, 16 * KIBIBYTE, PrgType::MirrorPrevious)
                    .build(),
            /* Nrom256 - A single long mapping. */
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

        let chr_rom_chunks = cartridge.chr_rom_chunks();
        let raw_pattern_tables = match chr_rom_chunks.len() {
            // Provide empty CHR RAM if the cartridge doesn't provide any CHR ROM.
            0 => [MappedArray::<4>::empty(), MappedArray::<4>::empty()],
            1 => split_chr_chunk(&*chr_rom_chunks[0]),
            n => {
                return Err(format!(
                    "CHR ROM size must be 0K or 8K for mapper 0, but was {}K",
                    8 * n
                ))
            }
        };

        let (raw_chr_memory, chr_type) = match chr_rom_chunks.len() {
            // Provide empty CHR RAM if the cartridge doesn't provide any CHR ROM.
            0 => (vec![0; 8 * KIBIBYTE], ChrType::Ram { bank_index: 0 }),
            1 => (cartridge.chr_rom(), ChrType::Rom { bank_index: 0 }),
            n => {
                return Err(format!(
                    "CHR ROM size must be 0K or 8K for mapper 0, but was {}K",
                    8 * n
                ))
            }
        };

        let chr_memory = ChrMemory::builder()
            .raw_memory(raw_chr_memory)
            .bank_count(1)
            .bank_size(8 * KIBIBYTE)
            .add_window(0x0000, 0x1FFF, 8 * KIBIBYTE, chr_type)
            .build();

        Ok(Mapper0 {
            prg_memory,
            chr_memory,
            raw_pattern_tables,
            name_table_mirroring: cartridge.name_table_mirroring(),
            is_chr_writable: chr_rom_chunks.is_empty(),
        })
    }
}

impl Mapper for Mapper0 {
    fn name_table_mirroring(&self) -> NameTableMirroring {
        self.name_table_mirroring
    }

    #[inline]
    fn prg_memory(&self) -> &PrgMemory {
        &self.prg_memory
    }

    fn chr_memory(&self) -> &ChrMemory {
        &self.chr_memory
    }

    fn chr_memory_mut(&mut self) -> &mut ChrMemory {
        &mut self.chr_memory
    }

    #[inline]
    fn is_chr_writable(&self) -> bool {
        self.is_chr_writable
    }

    #[inline]
    fn raw_pattern_table(&self, side: PatternTableSide) -> &RawPatternTable {
        &self.raw_pattern_tables[side as usize]
    }

    fn chr_bank_chunks(&self) -> Vec<Vec<Chunk>> {
        // Mapper 0 has no CHR banks.
        Vec::new()
    }

    fn write_to_prg_memory(&mut self, _address: CpuAddress, _value: u8) {
        // Does nothing for mapper 0.
    }

    fn chr_rom_bank_string(&self) -> String {
        "(Fixed)".to_string()
    }
}
