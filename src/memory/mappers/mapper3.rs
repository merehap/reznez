use crate::cartridge::Cartridge;
use crate::memory::cpu::cpu_address::CpuAddress;
use crate::memory::cpu::cartridge_space::CartridgeSpace;
use crate::memory::mapper::*;
use crate::ppu::name_table::name_table_mirroring::NameTableMirroring;
use crate::ppu::pattern_table::PatternTableSide;
use crate::util::bit_util::get_bit;
use crate::util::mapped_array::Chunk;

const EMPTY_CHR_CHUNK: [u8; 0x2000] = [0; 0x2000];
const BANK_SELECT_START: CpuAddress = CpuAddress::new(0x8000);

// CNROM
pub struct Mapper3 {
    cartridge_space: CartridgeSpace,
    raw_pattern_tables: [RawPatternTablePair; 4],
    selected_chr_bank: ChrBankId,
    name_table_mirroring: NameTableMirroring,
}

impl Mapper3 {
    pub fn new(cartridge: &Cartridge) -> Result<Mapper3, String> {
        let prg_rom_chunks = cartridge.prg_rom_chunks();
        let cartridge_space = match prg_rom_chunks.len() {
            1 => CartridgeSpace::single_bank_mirrored(prg_rom_chunks[0].clone()),
            2 => CartridgeSpace::single_bank(Box::new(cartridge.prg_rom().try_into().unwrap())),
            c => {
                return Err(format!(
                    "PRG ROM size must be 16K or 32K for this mapper, but was {}K",
                    16 * c,
                ))
            }
        };

        let chr_chunk_count = cartridge.chr_rom_chunks().len();
        if chr_chunk_count > 4 {
            return Err(format!(
                "Max CHR chunks for Mapper 3 is 4, but found {}.",
                chr_chunk_count,
            ));
        }

        let mut chunk_iter = cartridge.chr_rom_chunks().iter();
        let raw_pattern_tables = [
            split_chr_chunk(&**chunk_iter.next().unwrap_or(&Box::new(EMPTY_CHR_CHUNK))),
            split_chr_chunk(&**chunk_iter.next().unwrap_or(&Box::new(EMPTY_CHR_CHUNK))),
            split_chr_chunk(&**chunk_iter.next().unwrap_or(&Box::new(EMPTY_CHR_CHUNK))),
            split_chr_chunk(&**chunk_iter.next().unwrap_or(&Box::new(EMPTY_CHR_CHUNK))),
        ];
        let name_table_mirroring = cartridge.name_table_mirroring();
        Ok(Mapper3 {
            cartridge_space,
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

    fn cartridge_space(&self) -> &CartridgeSpace {
        &self.cartridge_space
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

    fn write_to_cartridge_space(&mut self, cpu_address: CpuAddress, value: u8) {
        if cpu_address >= BANK_SELECT_START {
            //println!("Switching to bank {} ({}). Address: {}.", value % 4, value, cpu_address);
            self.selected_chr_bank = ChrBankId::from_u8(value);
        }
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
