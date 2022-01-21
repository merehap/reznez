use crate::cartridge::Cartridge;
use crate::cpu::memory::Memory as CpuMem;
use crate::util::mapped_array::MemoryMappings;

pub struct Mapper0 {
    cartridge: Cartridge,
    mappings: Mappings,
}

impl Mapper0 {
    pub fn new(cartridge: Cartridge) -> Result<Mapper0, String> {
        let prg_rom = cartridge.prg_rom();
        let mut cpu_mappings = MemoryMappings::new();
        match cartridge.prg_rom_chunk_count() {
            /* Nrom128 - Mirrored mappings. */
            1 => {
                cpu_mappings.add_mapping(prg_rom[0x0..0x4000].into(), 0x8000)?;
                cpu_mappings.add_mapping(prg_rom[0x0..0x4000].into(), 0xC000)?;
            },
            /* Nrom256 - A single long mapping. */
            2 => {
                cpu_mappings.add_mapping(prg_rom[0x0..0x8000].into(), 0x8000)?;
            },
            c => return Err(format!(
                     "PRG ROM size must be 16K or 32K for mapper 0, but was {}K",
                     16 * u16::from(c),
                 )),
        };

        if cartridge.chr_rom_chunk_count() > 1 {
            return Err(format!(
                "CHR ROM size must be 0K or 8K for mapper 0, but was {}K",
                8 * cartridge.chr_rom_chunk_count()
            ));
        }

        let chr_rom = cartridge.chr_rom();
        let mut ppu_mappings = MemoryMappings::new();
        if !chr_rom.is_empty() {
            ppu_mappings.add_mapping(chr_rom[0x0..0x2000].into(), 0x0)?;
        }

        let mappings = Mappings {cpu_mappings, ppu_mappings};
        Ok(Mapper0 {cartridge, mappings})
    }

    pub fn current_mappings(&self, _: &CpuMem) -> Mappings {
        self.mappings.clone()
    }
}

#[derive(Clone)]
pub struct Mappings {
    pub cpu_mappings: MemoryMappings,
    pub ppu_mappings: MemoryMappings,
}
