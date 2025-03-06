use log::warn;

use crate::cartridge::cartridge::Cartridge;
use crate::util::unit::KIBIBYTE;

#[allow(non_camel_case_types)]
#[allow(clippy::upper_case_acronyms)]
#[derive(PartialEq, Eq, Debug)]
pub enum Board {
    Unknown,

    SAROM,
    SBROM,
    // Including Sc1rom.
    SCROM_SL1ROM,
    SEROM,
    // Only the 128KiB PRGROM variants of SFROM.
    SFROM128,
    // Includes SF1ROM and SFEXPROM.
    SFROM256,
    SGROM,
    SGROM_SMROM,
    // Includes SHR1ROM.
    SHROM,
    SIROM,
    SJROM,
    SKROM,
    // SLROM, SL1ROM (except 64KiB PRG), SL2ROM, SL3ROM, SLRROM.
    // This can be broken down further if desired.
    SLROM,
    SNROM,
    SOROM,
    SUROM,
    SXROM,
    SZROM,
}


impl Board {
    pub fn from_cartridge(cartridge: &Cartridge) -> Self {
        let prg_rom_size = cartridge.prg_rom_size() / KIBIBYTE;
        let prg_ram_size = cartridge.prg_ram_size() / KIBIBYTE;
        let prg_nvram_size = cartridge.prg_nvram_size() / KIBIBYTE;
        let chr_rom_size = cartridge.chr_rom_size() / KIBIBYTE;
        let mut chr_ram_size = cartridge.chr_ram_size() / KIBIBYTE;
        // FIXME: Hack for ROMs that don't specify CHR sizes.
        if chr_rom_size == 0 && chr_ram_size == 0 {
            chr_ram_size = 8;
        }

        use Board::*;
        let board = match (prg_rom_size, prg_ram_size, prg_nvram_size, chr_rom_size, chr_ram_size) {
            (64             ,  8, 0, 16 | 32 | 64, 0) => SAROM,
            (64             ,  0, 0, 16 | 32 | 64, 0) => SBROM,
            (64             ,  0, 0,          128, 0) => SCROM_SL1ROM,
            (32             ,  0, 0, 16 | 32 | 64, 0) => SEROM,
            (128            ,  0, 0, 16 | 32 | 64, 0) => SFROM128,
            (256            ,  0, 0, 16 | 32 | 64, 0) => SFROM256,
            (128            ,  0, 0,            8, 0) => SGROM,
            (128 | 256      ,  0, 0,            0, 8) => SGROM,
            (256            ,  0, 0,            8, 0) => SGROM_SMROM,
            (32             ,  0, 0,          128, 0) => SHROM,
            (32             ,  8, 0, 16 | 32 | 64, 0) => SIROM,
            (128 | 256      ,  8, 0, 16 | 32 | 64, 0) => SJROM,
            (128 | 256      ,  8, 0,          128, 0) => SKROM,
            (128 | 256      ,  0, 0,          128, 0) => SLROM,
            (128 | 256      ,  8, 0,            8, 0) => SNROM,
            (128 | 256      ,  8, 0,            0, 8) => SNROM,
            (128 | 256      , 16, 0,            8, 0) => SOROM,
            (128 | 256      , 16, 0,            0, 8) => SOROM,
            (      512      ,  8, 0,            8, 0) => SUROM,
            (      512      ,  8, 0,            0, 8) => SUROM,
            (128 | 256 | 512, 32, 0,            8, 0) => SXROM,
            (128 | 256 | 512, 32, 0,            0, 8) => SXROM,
            (128 | 256      ,  8, 8, 16 | 32 | 64, 0) => SZROM,
            _ => {
                warn!("Unknown MMC1 board: ({prg_rom_size}KiB, {prg_ram_size}KiB, {chr_rom_size}KiB, {chr_ram_size}KiB)");
                Unknown
            }
        };

        if matches!(board, SEROM | SHROM | SNROM | SOROM | SUROM | SXROM ) {
            todo!("MMC1 {board:?} is not yet supported.");
        }

        board
    }
}