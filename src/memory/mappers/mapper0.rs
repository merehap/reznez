use crate::cartridge::Cartridge;
use crate::memory::mapper::*;

const PRG_ROM_SIZE: usize = 0x8000;

pub struct Mapper0 {
    prg_rom: Box<[u8; 0x8000]>,
    chr_rom: Box<[u8; 0x2000]>,
}

impl Mapper0 {
    pub fn new(cartridge: Cartridge) -> Result<Mapper0, String> {
        let mut prg_rom = Box::new([0; PRG_ROM_SIZE]);
        match cartridge.prg_rom_chunk_count() {
            /* Nrom128 - Mirrored mappings. */
            1 => {
                for (i, byte) in cartridge.prg_rom().iter().enumerate().take(0x4000) {
                    prg_rom[i] = *byte;
                    prg_rom[i + 0x4000] = *byte;
                }
            },
            /* Nrom256 - A single long mapping. */
            2 => {
                for (i, byte) in cartridge.prg_rom().iter().enumerate().take(0x8000) {
                    prg_rom[i] = *byte;
                }
            },
            c => return Err(format!(
                     "PRG ROM size must be 16K or 32K for mapper 0, but was {}K",
                     16 * u16::from(c),
                 )),
        }

        if cartridge.chr_rom_chunk_count() > 1 {
            return Err(format!(
                "CHR ROM size must be 0K or 8K for mapper 0, but was {}K",
                8 * cartridge.chr_rom_chunk_count()
            ));
        }

        let chr_rom =
            if cartridge.chr_rom().is_empty() {
                // Provide empty CHR ROM if the cartridge doesn't provide any.
                Box::new([0; 0x2000])
            } else {
                Box::new(cartridge.chr_rom()[0x0..0x2000].try_into().unwrap())
            };

        Ok(Mapper0 {prg_rom, chr_rom})
    }
}

impl Mapper for Mapper0 {
    fn prg_rom(&self) -> &[u8; 0x8000] {
        self.prg_rom.as_ref()
    }

    fn raw_pattern_table(&self) -> &[u8; PATTERN_TABLE_SIZE] {
        self.chr_rom.as_ref()
    }

    fn raw_pattern_table_mut(&mut self) -> &mut [u8; PATTERN_TABLE_SIZE] {
        self.chr_rom.as_mut()
    }
}