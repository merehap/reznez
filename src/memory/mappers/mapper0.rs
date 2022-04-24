use crate::cartridge::Cartridge;
use crate::memory::cpu::cpu_address::CpuAddress;
use crate::memory::mapper::*;
use crate::ppu::name_table::name_table_mirroring::NameTableMirroring;
use crate::ppu::pattern_table::PatternTableSide;
use crate::util::mapped_array::{MappedArray, Chunk};

// NROM
pub struct Mapper0 {
    prg_rom: MappedArray<32>,
    raw_pattern_tables: RawPatternTablePair,
    name_table_mirroring: NameTableMirroring,
    is_chr_writable: bool,
}

impl Mapper0 {
    pub fn new(cartridge: &Cartridge) -> Result<Mapper0, String> {
        let prg_rom_chunks = cartridge.prg_rom_chunks();
        let prg_rom =
            match prg_rom_chunks.len() {
                /* Nrom128 - Mirrored mappings. */
                1 => MappedArray::<32>::mirror_half(*prg_rom_chunks[0]),
                /* Nrom256 - A single long mapping. */
                2 => MappedArray::<32>::new::<0x8000>(cartridge.prg_rom().try_into().unwrap()),
                c => return Err(format!(
                         "PRG ROM size must be 16K or 32K for this mapper, but was {}K",
                         16 * c,
                     )),
            };

        let chr_rom_chunks = cartridge.chr_rom_chunks();
        let raw_pattern_tables =
            match chr_rom_chunks.len() {
                // Provide empty CHR RAM if the cartridge doesn't provide any CHR ROM.
                0 => [MappedArray::<4>::empty(), MappedArray::<4>::empty()],
                1 => split_chr_chunk(&*chr_rom_chunks[0]),
                n => return Err(format!(
                         "CHR ROM size must be 0K or 8K for mapper 0, but was {}K",
                         8 * n
                     )),
            };

        let name_table_mirroring = cartridge.name_table_mirroring();
        let is_chr_writable = chr_rom_chunks.is_empty();
        Ok(Mapper0 {prg_rom, raw_pattern_tables, name_table_mirroring, is_chr_writable})
    }
}

impl Mapper for Mapper0 {
    fn name_table_mirroring(&self) -> NameTableMirroring {
        self.name_table_mirroring
    }

    #[inline]
    fn prg_rom(&self) -> &MappedArray<32> {
        &self.prg_rom
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

    fn read_prg_ram(&self, _address: CpuAddress) -> u8 {
        // FIXME: Change to open bus behavior.
        0
    }

    fn write_to_cartridge_space(&mut self, _address: CpuAddress, _value: u8) {
        // Does nothing for mapper 0.
    }

    fn prg_rom_bank_string(&self) -> String {
        "(Fixed)".to_string()
    }

    fn chr_rom_bank_string(&self) -> String {
        "(Fixed)".to_string()
    }
}
