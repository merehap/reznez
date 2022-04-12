use crate::cartridge::Cartridge;
use crate::memory::cpu::cpu_address::CpuAddress;
use crate::memory::mapper::*;
use crate::ppu::name_table::name_table_mirroring::NameTableMirroring;
use crate::ppu::pattern_table::PatternTableSide;
use crate::util::bit_util::get_bit;
use crate::util::mapped_array::{MappedArray, Chunk};

const EMPTY_CHR_CHUNK: [u8; 0x2000] = [0; 0x2000];

// CNROM
pub struct Mapper3 {
    prg_rom: MappedArray<32>,
    raw_pattern_tables: [[MappedArray<4>; 2]; 4],
    selected_chr_bank: ChrBankId,
    name_table_mirroring: NameTableMirroring,
}

impl Mapper3 {
    pub fn new(cartridge: &Cartridge) -> Result<Mapper3, String> {
        let prg_rom_chunks = cartridge.prg_rom_chunks();
        let prg_rom =
            match prg_rom_chunks.len() {
                1 => MappedArray::<32>::mirror_half(*prg_rom_chunks[0]),
                2 => MappedArray::<32>::new::<0x8000>(cartridge.prg_rom().try_into().unwrap()),
                c => return Err(format!(
                         "PRG ROM size must be 16K or 32K for this mapper, but was {}K",
                         16 * c,
                     )),
            };

        let chr_chunk_count = cartridge.chr_rom_chunks().len();
        if chr_chunk_count > 4 {
            return Err(format!(
                "Max CHR chunks for Mapper 3 is 4, but found {}.",
                chr_chunk_count,
            ));
        }

        let mut chunk_iter = cartridge.chr_rom_chunks().iter();
        let raw_pattern_tables =
            [
                split_chr_chunk(&**chunk_iter.next().unwrap_or(&Box::new(EMPTY_CHR_CHUNK))),
                split_chr_chunk(&**chunk_iter.next().unwrap_or(&Box::new(EMPTY_CHR_CHUNK))),
                split_chr_chunk(&**chunk_iter.next().unwrap_or(&Box::new(EMPTY_CHR_CHUNK))),
                split_chr_chunk(&**chunk_iter.next().unwrap_or(&Box::new(EMPTY_CHR_CHUNK))),
            ];
        let name_table_mirroring = cartridge.name_table_mirroring();
        Ok(Mapper3 {
            prg_rom,
            raw_pattern_tables,
            selected_chr_bank: ChrBankId::Zero,
            name_table_mirroring,
        })
    }
}

impl Mapper for Mapper3 {
    fn name_table_mirroring(&self) -> NameTableMirroring {
        self.name_table_mirroring
    }

    fn prg_rom(&self) -> &MappedArray<32> {
        &self.prg_rom
    }

    #[inline]
    fn is_chr_writable(&self) -> bool {
        false
    }

    fn raw_pattern_table(&self, side: PatternTableSide) -> &MappedArray<4> {
        &self.raw_pattern_tables[self.selected_chr_bank as usize][side as usize]
    }

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

    fn write_to_cartridge_space(&mut self, _cpu_address: CpuAddress, value: u8) {
        //println!("Switching to bank {} ({}). Address: {}.", value % 4, value, cpu_address);
        self.selected_chr_bank = ChrBankId::from_u8(value);
    }

    fn prg_rom_bank_string(&self) -> String {
        "(Fixed)".to_string()
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
    pub fn from_u8(value: u8) -> ChrBankId {
        match (get_bit(value, 6), get_bit(value, 7)) {
            (false, false) => ChrBankId::Zero,
            (false, true ) => ChrBankId::One,
            (true , false) => ChrBankId::Two,
            (true , true ) => ChrBankId::Three,
        }
    }
}
