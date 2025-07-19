use crate::mapper::CiramSide;
use crate::memory::memory::Memory;
use crate::memory::ppu::chr_memory::PeekSource;
use crate::ppu::palette::util;

use super::rgb::Rgb;

pub struct BankColorAssigner {
    rom_spectrum: Vec<Rgb>,
    ram_greyscale: Vec<Rgb>,
}

impl BankColorAssigner {
    pub fn new(memory: &Memory) -> Self {
        Self {
            rom_spectrum: util::spectrum(memory.chr_rom_bank_count()),
            ram_greyscale: util::greyscale(memory.chr_ram_bank_count()),
        }
    }

    pub fn rgb_for_source(&self, source: PeekSource) -> Rgb {
        match source {
            // Middle brightness colors (the standard color wheel)
            PeekSource::Rom(bank_index) => {
                let bank_index = bank_index.to_raw() as usize % self.rom_spectrum.len();
                self.rom_spectrum[bank_index]
            }
            // Greyscale
            PeekSource::Ram(bank_index) => {
                let bank_index = bank_index.to_raw() as usize % self.ram_greyscale.len();
                self.ram_greyscale[bank_index]
            }
            // Midnight Blue
            PeekSource::Ciram(CiramSide::Left) => Rgb::new(0x26, 0x00, 0x4D),
            // Dark Scarlet
            PeekSource::Ciram(CiramSide::Right) => Rgb::new(0x4D, 0x00, 0x26),
            // Dark Sienna
            PeekSource::SaveRam => Rgb::new(0x4D, 0x26, 0x00),
            // Lincoln Green
            PeekSource::PaletteTable => Rgb::new(0x26, 0x4D, 0x00),
            // Forest Green
            PeekSource::ExtendedRam => Rgb::new(0x00, 0x4D, 0x26),
            // Oxford Blue
            PeekSource::FillModeTile => Rgb::new(0x00, 0x26, 0x4D),
        }
    }
}