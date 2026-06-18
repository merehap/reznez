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

        match index {
            0x00 => self.universal_background_color = self.ram[index].peek().into(),
            0x01..=0x03 => self.background_palettes[0].set_color(index - 0x01, self.ram[index].peek().into()),
            0x05..=0x07 => self.background_palettes[1].set_color(index - 0x05, self.ram[index].peek().into()),
            0x09..=0x0B => self.background_palettes[2].set_color(index - 0x09, self.ram[index].peek().into()),
            0x0D..=0x0F => self.background_palettes[3].set_color(index - 0x0D, self.ram[index].peek().into()),
            0x11..=0x13 => self.sprite_palettes[0].set_color(index - 0x11, self.ram[index].peek().into()),
            0x15..=0x17 => self.sprite_palettes[1].set_color(index - 0x15, self.ram[index].peek().into()),
            0x19..=0x1B => self.sprite_palettes[2].set_color(index - 0x19, self.ram[index].peek().into()),
            0x1D..=0x1F => self.sprite_palettes[3].set_color(index - 0x1D, self.ram[index].peek().into()),
            // TODO: Verify if any of these do anything. 0x10 may be a mirror of 0x00.
            0x04 | 0x08 | 0x0C | 0x10 | 0x14 | 0x18 | 0x1C => {}
            0x20.. => unreachable!(),
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
