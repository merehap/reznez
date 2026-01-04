use crate::mapper::CiramSide;
use crate::memory::memory::Bus;
use crate::memory::ppu::chr_memory::PeekSource;
use crate::ppu::palette::util;

use super::rgb::Rgb;

pub struct BankColorAssigner {
    rom_spectrum: Vec<Rgb>,
    ram_greyscale: Vec<Rgb>,
}

impl BankColorAssigner {
    pub fn new(bus: &Bus) -> Self {
        Self {
            rom_spectrum: util::spectrum(bus.chr_rom_bank_count()),
            ram_greyscale: util::greyscale(bus.chr_ram_bank_count()),
        }
    }

    pub fn rgb_for_source(&self, source: PeekSource) -> Rgb {
        match source {
            // Middle brightness colors (the standard color wheel)
            PeekSource::Rom(bank_number) => {
                let bank_number = bank_number.to_raw() as usize % self.rom_spectrum.len();
                self.rom_spectrum[bank_number]
            }
            // Greyscale
            PeekSource::Ram(bank_number) => {
                let bank_number = bank_number.to_raw() as usize % self.ram_greyscale.len();
                self.ram_greyscale[bank_number]
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
            PeekSource::MapperCustom { page_number: 0, .. } => Rgb::new(0x00, 0x4D, 0x26),
            // Oxford Blue
            PeekSource::MapperCustom { page_number: 1, .. } => Rgb::new(0x00, 0x26, 0x4D),
            PeekSource::MapperCustom { page_number: _, .. } =>
                todo!("No currently supported mappers have more than two pages of custom memory."),
        }
    }
}