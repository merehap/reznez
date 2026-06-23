use ux::u5;

use crate::memory::ppu::chr_memory::{PeekSource, PpuPeek};
use crate::memory::ppu::ppu_address::PaletteRamIndex;
use crate::ppu::palette::palette_table_index::PaletteTableIndex;
use crate::ppu::register::ppu_registers::PpuRegisters;
use crate::ppu::palette::color::Color;
use crate::ppu::palette::palette::Palette;

const PALETTE_RAM_SIZE: usize = 0x20;
const INITIAL_PALETTE_DATA: [u8; PALETTE_RAM_SIZE] = [
    0x09, 0x01, 0x00, 0x01, 0x00, 0x02, 0x02, 0x0D, 0x08, 0x10, 0x08, 0x24, 0x00, 0x00, 0x04, 0x2C,
    0x09, 0x01, 0x34, 0x03, 0x00, 0x04, 0x00, 0x14, 0x08, 0x3A, 0x00, 0x02, 0x00, 0x20, 0x2C, 0x08,
];

// See https://wiki.nesdev.org/w/index.php?title=PPU_palettes#Memory_Map
pub struct PaletteRam {
    backdrop_color: Color,             // 0x00 and 0x10
    unused_colors: [Color; 3],         // 0x04, 0x08, 0x0C (and their mirrors: 0x14, 0x18, 0x1C)
    background_palettes: [Palette; 4], // 0x01..=0x0F (except unpaletted values)
    sprite_palettes: [Palette; 4],     // 0x11..=0x1F (except unpaletted values)
}

impl PaletteRam {
    pub fn new() -> Self {
        let mut palette_ram = PaletteRam {
            backdrop_color: Color::BLACK,
            unused_colors: [Color::BLACK; 3],
            background_palettes: [Palette::ALL_BLACK; 4],
            sprite_palettes: [Palette::ALL_BLACK; 4],
        };

        for (i, &data) in INITIAL_PALETTE_DATA.iter().enumerate() {
            let i = PaletteRamIndex::new(u5::new(i as u8));
            palette_ram.write(i, data);
        }

        palette_ram
    }

    pub fn peek(&self, regs: &PpuRegisters, index: PaletteRamIndex) -> PpuPeek {
        let mut color = match index {
            PaletteRamIndex::BackdropColor => self.backdrop_color,
            PaletteRamIndex::Unused1 => self.unused_colors[0],
            PaletteRamIndex::Unused2 => self.unused_colors[1],
            PaletteRamIndex::Unused3 => self.unused_colors[2],
            PaletteRamIndex::Background(table_index, palette_index) => {
                self.background_palette(table_index).color(palette_index as usize)
            }
            PaletteRamIndex::Sprite(table_index, palette_index) => {
                self.sprite_palette(table_index).color(palette_index as usize)
            }
        };
        if regs.mask().greyscale_enabled() {
            color = color.to_greyscale();
        }

        PpuPeek::new(color.to_u6().into(), PeekSource::PaletteTable)
    }

    pub fn write(&mut self, index: PaletteRamIndex, value: u8) {
        let color: Color = (value & 0b0011_1111).into();
        match index {
            PaletteRamIndex::BackdropColor => self.backdrop_color = color,
            PaletteRamIndex::Unused1 => self.unused_colors[0] = color,
            PaletteRamIndex::Unused2 => self.unused_colors[1] = color,
            PaletteRamIndex::Unused3 => self.unused_colors[2] = color,
            PaletteRamIndex::Background(table_index, palette_index) => {
                self.background_palettes[table_index as usize].set_color(palette_index as usize, color);
            }
            PaletteRamIndex::Sprite(table_index, palette_index) => {
                self.sprite_palettes[table_index as usize].set_color(palette_index as usize, color);
            }
        }
    }

    #[inline]
    pub fn backdrop_color(&self) -> Color {
        self.backdrop_color
    }

    #[inline]
    pub fn background_palette(&self, number: PaletteTableIndex) -> Palette {
        self.background_palettes[number as usize]
    }

    #[inline]
    pub fn sprite_palette(&self, number: PaletteTableIndex) -> Palette {
        self.sprite_palettes[number as usize]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blank_first_bits() {
        let ppu_regs = PpuRegisters::new();
        let mut palette_ram = PaletteRam::new();
        let index = PaletteRamIndex::new(u5::new(12));
        assert_eq!(palette_ram.peek(&ppu_regs, index).value(), 0b0000_0000);
        palette_ram.write(index                              , 0b1110_1010);
        assert_eq!(palette_ram.peek(&ppu_regs, index).value(), 0b0010_1010);
    }
}
