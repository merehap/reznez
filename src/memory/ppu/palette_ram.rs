use crate::memory::ppu::chr_memory::{PeekSource, PpuPeek};
use crate::memory::primitives::masked_byte::MaskedByte;
use crate::ppu::palette::palette_table_index::PaletteTableIndex;
use crate::ppu::register::ppu_registers::PpuRegisters;
use crate::ppu::palette::color::Color;
use crate::ppu::palette::palette::Palette;

const PALETTE_RAM_SIZE: usize = 0x20;
const INITIAL_PALETTE_DATA: [u8; PALETTE_RAM_SIZE] = [
    0x09, 0x01, 0x00, 0x01, 0x00, 0x02, 0x02, 0x0D, 0x08, 0x10, 0x08, 0x24, 0x00, 0x00, 0x04, 0x2C,
    0x09, 0x01, 0x34, 0x03, 0x00, 0x04, 0x00, 0x14, 0x08, 0x3A, 0x00, 0x02, 0x00, 0x20, 0x2C, 0x08,
];

pub struct PaletteRam {
    // First two bits are always 0 for palette RAM bytes.
    // See https://wiki.nesdev.org/w/index.php?title=PPU_palettes#Memory_Map
    ram: [MaskedByte<0b0011_1111>; PALETTE_RAM_SIZE],

    universal_background_color: Color,
    background_palettes: [Palette; 4],
    sprite_palettes: [Palette; 4],
}

impl PaletteRam {
    pub fn new() -> Self {
        let mut palette_ram = PaletteRam {
            ram: [MaskedByte::new(0); PALETTE_RAM_SIZE],
            universal_background_color: Color::BLACK,
            background_palettes: [Palette::ALL_BLACK; 4],
            sprite_palettes: [Palette::ALL_BLACK; 4],
        };

        for (i, &data) in INITIAL_PALETTE_DATA.iter().enumerate() {
            palette_ram.write(i as u32, data);
        }

        palette_ram
    }

    pub fn peek(&self, regs: &PpuRegisters, index: u32) -> PpuPeek {
        let mut value = self.ram[index as usize].peek();
        if regs.mask().greyscale_enabled() {
            value &= 0b1111_0000;
        }

        PpuPeek::new(value, PeekSource::PaletteTable)
    }

    pub fn write(&mut self, index: u32, value: u8) {
        let index = index as usize;
        self.ram[index].write(value);
        let color = self.ram[index].peek().into();

        let palettes = if index < 0x10 {
            &mut self.background_palettes
        } else {
            &mut self.sprite_palettes
        };

        let offset = index & 0b1111;
        match offset {
            0x00 => self.universal_background_color = color,
            0x04 | 0x08 | 0x0C => { /* Seemingly do nothing. */ }
            0x01..=0x03 => palettes[0].set_color(offset - 0x01, color),
            0x05..=0x07 => palettes[1].set_color(offset - 0x05, color),
            0x09..=0x0B => palettes[2].set_color(offset - 0x09, color),
            0x0D..=0x0F => palettes[3].set_color(offset - 0x0D, color),
            0x10.. => unreachable!(),
        }
    }

    pub fn to_slice(&self) -> &[MaskedByte<0b0011_1111>; PALETTE_RAM_SIZE] {
        &self.ram
    }

    #[inline]
    pub fn universal_background_color(&self) -> Color {
        self.universal_background_color
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
        assert_eq!(palette_ram.peek(&ppu_regs, 12).value(), 0b0000_0000);
        palette_ram.write(12, 0b1110_1010);
        assert_eq!(palette_ram.peek(&ppu_regs, 12).value(), 0b0010_1010);
    }
}
