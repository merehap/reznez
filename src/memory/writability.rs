use crate::memory::cpu::prg_memory::RomRamMode;

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Writability {
    Rom,
    Ram,
    RomRam,
}

impl Writability {
    pub fn is_writable(self, rom_ram_mode: RomRamMode) -> bool {
        match (self, rom_ram_mode) {
            (Writability::Rom, _) => false,
            (Writability::Ram, _) => true,
            (Writability::RomRam, RomRamMode::Rom) => false,
            (Writability::RomRam, RomRamMode::Ram) => true,
        }
    }
}
