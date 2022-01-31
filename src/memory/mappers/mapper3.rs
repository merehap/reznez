use arr_macro::arr;

use crate::cartridge::Cartridge;
use crate::memory::cpu_address::CpuAddress;
use crate::memory::mapper::*;
use crate::ppu::pattern_table::PatternTableSide;
use crate::util::bit_util::get_bit;
use crate::util::mapped_array::{MappedArray, MappedArrayMut};

// CNROM
pub struct Mapper3 {
    prg_rom: Box<[u8; 0x8000]>,
    chr_rom_banks: [Box<[u8; 0x2000]>; 4],
    selected_chr_bank: ChrBankId,
}

impl Mapper3 {
    pub fn new(cartridge: Cartridge) -> Result<Mapper3, String> {
        let mut prg_rom = Box::new([0; PRG_ROM_SIZE]);
        let prg_rom_chunks = cartridge.prg_rom_chunks();
        match prg_rom_chunks.len() {
            1 => {
                prg_rom[0x0000..=0x3FFF].copy_from_slice(prg_rom_chunks[0].as_ref());
                prg_rom[0x4000..=0x7FFF].copy_from_slice(prg_rom_chunks[0].as_ref());
            },
            2 => prg_rom.copy_from_slice(&cartridge.prg_rom()),
            c => return Err(format!(
                     "PRG ROM size must be 16K or 32K for this mapper, but was {}K",
                     16 * c,
                 )),
        }

        let chr_chunk_count = cartridge.chr_rom_chunks().len();
        if chr_chunk_count > 4 {
            return Err(format!(
                "Max CHR chunks for Mapper 3 is 4, but found {}.",
                chr_chunk_count,
            ));
        }

        let mut chunk_iter = cartridge.chr_rom_chunks().into_iter();
        let chr_rom_banks = arr![chunk_iter.next().unwrap().clone(); 4];

        Ok(Mapper3 {
            prg_rom,
            chr_rom_banks,
            selected_chr_bank: ChrBankId::Zero,
        })
    }
}

impl Mapper for Mapper3 {
    fn prg_rom(&self) -> MappedArray<'_, 32> {
        MappedArray::new(self.prg_rom.as_ref())
    }

    fn raw_pattern_table(&self, side: PatternTableSide) -> MappedArray<'_, 4> {
        let bank = self.chr_rom_banks[self.selected_chr_bank as usize].as_ref();
        let (start, end) = side.to_start_end();
        MappedArray::new::<PATTERN_TABLE_SIZE>((&bank[start..end]).try_into().unwrap())
    }

    fn raw_pattern_table_mut(&mut self, side: PatternTableSide) -> MappedArrayMut<'_, 4> {
        let bank = self.chr_rom_banks[self.selected_chr_bank as usize].as_mut();
        let (start, end) = side.to_start_end();
        MappedArrayMut::new::<PATTERN_TABLE_SIZE>((&mut bank[start..end]).try_into().unwrap())
    }

    fn read_prg_ram(&self, _: CpuAddress) -> u8 {
        self.selected_chr_bank as u8
    }

    fn write_to_cartridge_space(&mut self, _: CpuAddress, value: u8) {
        println!("Switching to bank {}.", value);
        self.selected_chr_bank = ChrBankId::from_u8(value);
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
