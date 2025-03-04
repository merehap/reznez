use crate::cartridge::cartridge::Cartridge;
use crate::util::unit::KIBIBYTE;

#[derive(PartialEq, Eq, Debug)]
pub enum Board {
    Unknown,
    // PRG ROM <= 256k, CHR RAM = 8k, PRG RAM = 8k
    Snrom,
    // PRG RAM = 16k
    Sorom,
    // 
    Surom,
    // PRG RAM = 32k
    Sxrom
}


impl Board {
    pub fn from_cartridge(cartridge: &Cartridge) -> Self {
        let prg_rom_size = cartridge.prg_rom_size() / KIBIBYTE;
        let prg_ram_size = cartridge.prg_ram_size() / KIBIBYTE;
        let chr_rom_size = cartridge.chr_rom_size() / KIBIBYTE;
        let chr_ram_size = cartridge.chr_ram_size() / KIBIBYTE;

        use Board::*;
        let board = match (prg_rom_size, prg_ram_size, chr_rom_size, chr_ram_size) {
            (128 | 256      ,  8, 8, 0) => Snrom,
            (128 | 256      ,  8, 0, 8) => Snrom,
            (128 | 256      , 16, 8, 0) => Sorom,
            (128 | 256      , 16, 0, 8) => Sorom,
            (      512      ,  8, 8, 0) => Surom,
            (      512      ,  8, 0, 8) => Surom,
            (128 | 256 | 512, 32, 8, 0) => Sxrom,
            (128 | 256 | 512, 32, 0, 8) => Sxrom,
            _ => Unknown,
        };

        if matches!(board, Snrom | Sorom | Sxrom) {
            todo!("MMC1 {board:?} is not yet supported.");
        }

        board
    }
}