use crate::cartridge::Cartridge;
use crate::memory::mapper::*;

pub struct Mapper0 {
    prg_rom: Box<[u8; 0x8000]>,
    chr_rom: Box<[u8; 0x2000]>,
}

impl Mapper0 {
    pub fn new(cartridge: Cartridge) -> Result<Mapper0, String> {
        let mut prg_rom = Box::new([0; PRG_ROM_SIZE]);
        let prg_rom_chunks = cartridge.prg_rom_chunks();
        match prg_rom_chunks.len() {
            /* Nrom128 - Mirrored mappings. */
            1 => {
                prg_rom[0x0000..=0x3FFF].copy_from_slice(prg_rom_chunks[0].as_ref());
                prg_rom[0x4000..=0x7FFF].copy_from_slice(prg_rom_chunks[0].as_ref());
            },
            /* Nrom256 - A single long mapping. */
            2 => prg_rom.copy_from_slice(&cartridge.prg_rom()),
            c => return Err(format!(
                     "PRG ROM size must be 16K or 32K for this mapper, but was {}K",
                     16 * c,
                 )),
        }

        let chr_rom_chunks = cartridge.chr_rom_chunks();
        let chr_rom =
            match chr_rom_chunks.len() {
                // Provide empty CHR ROM if the cartridge doesn't provide any.
                0 => Box::new([0; 0x2000]),
                1 => chr_rom_chunks[0].clone(),
                n => return Err(format!(
                         "CHR ROM size must be 0K or 8K for mapper 0, but was {}K",
                         8 * n
                     )),
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
