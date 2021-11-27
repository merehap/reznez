use crate::address::Address;
use crate::cartridge::INes;
use crate::memory::Memory;

pub struct Mapper0 {

}

impl Mapper0 {
    pub fn new() -> Mapper0 {
        Mapper0 {}
    }

    pub fn map(&self, ines: INes, memory: &mut Memory) -> Result<(), String> {
        let mut address = Address::new(0x8000);
        let high_source_index = match ines.prg_rom_chunk_count() {
            1 => /* Nrom128 */ 0,
            2 => /* Nrom256 */ 0x4000,
            c => return Err(format!(
                     "PRG ROM size must be 16K or 32K for mapper 0, but was {}K",
                     16 * (c as u16),
                 )),
        };

        let prg_rom = ines.prg_rom();

        let mut low_address = Address::new(0x8000);
        let mut high_address = Address::new(0xC000);
        for i in 0..0x4000 {
            memory[low_address] = prg_rom[i];
            // Copy high ROM (for NROM256) or mirror low ROM (for NROM128).
            memory[high_address] = prg_rom[high_source_index + i];
            low_address.inc();
            high_address.inc();
        }

        Ok(())
    }
}
