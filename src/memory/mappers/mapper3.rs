use crate::cartridge::Cartridge;
use crate::memory::cpu_address::CpuAddress;
use crate::memory::mapper::*;
use crate::util::bit_util::get_bit;

pub struct Mapper3 {
    prg_rom: Box<[u8; 0x8000]>,
    chr_rom_banks: [Box<[u8; 0x2000]>; 4],
    selected_bank: Bank,
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

        let bank = Box::new([0; 0x2000]);
        let mut chr_rom_banks = [bank.clone(), bank.clone(), bank.clone(), bank];
        for i in 0..chr_chunk_count {
            chr_rom_banks[i] = cartridge.chr_rom_chunks()[i].clone();
        }

        Ok(Mapper3 {
            prg_rom,
            chr_rom_banks,
            selected_bank: Bank::Zero,
        })
    }
}

impl Mapper for Mapper3 {
    fn prg_rom(&self) -> &[u8; 0x8000] {
        self.prg_rom.as_ref()
    }

    fn raw_pattern_table(&self) -> &[u8; PATTERN_TABLE_SIZE] {
        &self.chr_rom_banks[self.selected_bank as usize]
    }

    fn raw_pattern_table_mut(&mut self) -> &mut [u8; PATTERN_TABLE_SIZE] {
        &mut self.chr_rom_banks[self.selected_bank as usize]
    }

    fn read_prg_ram(&self, _: CpuAddress) -> u8 {
        self.selected_bank as u8
    }

    fn write_prg_ram(&mut self, _: CpuAddress, value: u8) {
        println!("Switching to bank {}.", value);
        self.selected_bank = Bank::from_u8(value);
    }
}

#[derive(Clone, Copy)]
enum Bank {
    Zero,
    One,
    Two,
    Three,
}

impl Bank {
    pub fn from_u8(value: u8) -> Bank {
        match (get_bit(value, 6), get_bit(value, 7)) {
            (false, false) => Bank::Zero,
            (false, true ) => Bank::One,
            (true , false) => Bank::Two,
            (true , true ) => Bank::Three,
        }
    }
}
