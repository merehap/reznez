use crate::cartridge::Cartridge;
use crate::memory::cpu::cpu_address::CpuAddress;
use crate::memory::cpu::prg_memory::{PrgMemory, PrgType};
use crate::memory::mapper::*;
use crate::ppu::name_table::name_table_mirroring::NameTableMirroring;
use crate::ppu::pattern_table::PatternTableSide;
use crate::util::bit_util::get_bit;
use crate::util::mapped_array::Chunk;
use crate::util::unit::KIBIBYTE;

const BANK_SELECT_START: CpuAddress = CpuAddress::new(0x8000);

// CNROM
pub struct Mapper3 {
    prg_memory: PrgMemory,
    raw_pattern_tables: Vec<RawPatternTablePair>,
    selected_chr_bank: ChrBankId,
    name_table_mirroring: NameTableMirroring,
}

impl Mapper3 {
    pub fn new(cartridge: &Cartridge) -> Result<Mapper3, String> {
        let prg_rom_chunks = cartridge.prg_rom_chunks();
        let prg_memory = match prg_rom_chunks.len() {
            1 => PrgMemory::builder()
                    .raw_memory(cartridge.prg_rom_chunks()[0].to_vec())
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

        let raw_pattern_tables = cartridge.chr_rom_chunks().iter()
            .map(|chunk| split_chr_chunk(chunk))
            .collect();

        Ok(Mapper3 {
            prg_memory,
            raw_pattern_tables,
            selected_chr_bank: ChrBankId::Zero,
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

    #[inline]
    fn is_chr_writable(&self) -> bool {
        false
    }

    fn raw_pattern_table(&self, side: PatternTableSide) -> &RawPatternTable {
        &self.raw_pattern_tables[self.selected_chr_bank as usize][side as usize]
    }

    #[rustfmt::skip]
    fn chr_bank_chunks(&self) -> Vec<Vec<Chunk>> {
        vec![
            self.raw_pattern_tables[0][PatternTableSide::Left as usize].to_chunks().to_vec(),
            self.raw_pattern_tables[0][PatternTableSide::Right as usize].to_chunks().to_vec(),
            self.raw_pattern_tables[1][PatternTableSide::Left as usize].to_chunks().to_vec(),
            self.raw_pattern_tables[1][PatternTableSide::Right as usize].to_chunks().to_vec(),
            self.raw_pattern_tables[2][PatternTableSide::Left as usize].to_chunks().to_vec(),
            self.raw_pattern_tables[2][PatternTableSide::Right as usize].to_chunks().to_vec(),
            self.raw_pattern_tables[3][PatternTableSide::Left as usize].to_chunks().to_vec(),
            self.raw_pattern_tables[3][PatternTableSide::Right as usize].to_chunks().to_vec(),
        ]
    }

    fn write_to_prg_memory(&mut self, cpu_address: CpuAddress, value: u8) {
        if cpu_address >= BANK_SELECT_START {
            //println!("Switching to bank {} ({}). Address: {}.", value % 4, value, cpu_address);
            self.selected_chr_bank = ChrBankId::from_u8(value);
        }
    }

    fn chr_rom_bank_string(&self) -> String {
        format!("{} of 4 [8 KiB banks]", self.selected_chr_bank as u8)
    }
}

#[derive(Clone, Copy)]
enum ChrBankId {
    Zero,
    One,
    Two,
    Three,
}

impl ChrBankId {
    #[rustfmt::skip]
    pub fn from_u8(value: u8) -> ChrBankId {
        match (get_bit(value, 6), get_bit(value, 7)) {
            (false, false) => ChrBankId::Zero,
            (false, true ) => ChrBankId::One,
            (true , false) => ChrBankId::Two,
            (true , true ) => ChrBankId::Three,
        }
    }
}
