use crate::memory::ppu::chr_memory::{PeekSource, PpuPeek};
use crate::memory::primitives::masked_byte::MaskedByte;
use crate::ppu::palette::palette_table_index::PaletteTableIndex;
use crate::ppu::palette::system_palette::SystemPalette;
use crate::ppu::register::ppu_registers::PpuRegisters;
use crate::ppu::palette::color::Color;
use crate::ppu::palette::palette::Palette;
use crate::ppu::palette::rgb::Rgb;

const PALETTE_RAM_SIZE: usize = 0x20;
const INITIAL_PALETTE_DATA: [u8; PALETTE_RAM_SIZE] = [
    0x09, 0x01, 0x00, 0x01, 0x00, 0x02, 0x02, 0x0D, 0x08, 0x10, 0x08, 0x24, 0x00, 0x00, 0x04, 0x2C,
    0x09, 0x01, 0x34, 0x03, 0x00, 0x04, 0x00, 0x14, 0x08, 0x3A, 0x00, 0x02, 0x00, 0x20, 0x2C, 0x08,
];

pub struct PaletteRam {
    // First two bits are always 0 for palette RAM bytes.
    // See https://wiki.nesdev.org/w/index.php?title=PPU_palettes#Memory_Map
    ram: [MaskedByte<0b0011_1111>; PALETTE_RAM_SIZE],

    universal_background_rgb: Rgb,
    background_palettes: [Palette; 4],
    sprite_palettes: [Palette; 4],

    system_palette: SystemPalette,
}

impl PaletteRam {
    pub fn new(system_palette: SystemPalette) -> PaletteRam {
        let ram = INITIAL_PALETTE_DATA.map(MaskedByte::new);

        let rgb = |raw_color: MaskedByte<0b0011_1111>| -> Rgb {
            system_palette.lookup_unemphasized_rgb(Color::from_u8(raw_color.peek()))
        };
        let universal_background_rgb = rgb(ram[0x00]);
        let background_palettes = [
            Palette::new([rgb(ram[0x01]), rgb(ram[0x02]), rgb(ram[0x03])]),
            Palette::new([rgb(ram[0x05]), rgb(ram[0x06]), rgb(ram[0x07])]),
            Palette::new([rgb(ram[0x09]), rgb(ram[0x0A]), rgb(ram[0x0B])]),
            Palette::new([rgb(ram[0x0D]), rgb(ram[0x0E]), rgb(ram[0x0F])]),
        ];
        let sprite_palettes = [
            Palette::new([rgb(ram[0x11]), rgb(ram[0x12]), rgb(ram[0x13])]),
            Palette::new([rgb(ram[0x15]), rgb(ram[0x16]), rgb(ram[0x17])]),
            Palette::new([rgb(ram[0x19]), rgb(ram[0x1A]), rgb(ram[0x1B])]),
            Palette::new([rgb(ram[0x1D]), rgb(ram[0x1E]), rgb(ram[0x1F])]),
        ];

        Self { ram, universal_background_rgb, background_palettes, sprite_palettes, system_palette }
    }

    pub fn peek(&self, regs: &PpuRegisters, index: u32) -> PpuPeek {
        let mut value = self.ram[index as usize].peek();
        if regs.mask().greyscale_enabled() {
            value &= 0b1111_0000;
        }

        PpuPeek::new(value, PeekSource::PaletteTable)
    }

    // TODO: Mask should probably be applied at render time.
    pub fn write(&mut self, index: u32, value: u8) {
        let index = index as usize;
        self.ram[index].write(value);

        let rgb = |raw_color: MaskedByte<0b0011_1111>| -> Rgb {
            self.system_palette.lookup_unemphasized_rgb(Color::from_u8(raw_color.peek()))
        };
        match index {
            0x00 => self.universal_background_rgb = rgb(self.ram[index]),
            0x01..=0x03 => self.background_palettes[0].set_rgb(index - 0x01, rgb(self.ram[index])),
            0x05..=0x07 => self.background_palettes[1].set_rgb(index - 0x05, rgb(self.ram[index])),
            0x09..=0x0B => self.background_palettes[2].set_rgb(index - 0x09, rgb(self.ram[index])),
            0x0D..=0x0F => self.background_palettes[3].set_rgb(index - 0x0D, rgb(self.ram[index])),
            0x11..=0x13 => self.sprite_palettes[0].set_rgb(index - 0x11, rgb(self.ram[index])),
            0x15..=0x17 => self.sprite_palettes[1].set_rgb(index - 0x15, rgb(self.ram[index])),
            0x19..=0x1B => self.sprite_palettes[2].set_rgb(index - 0x19, rgb(self.ram[index])),
            0x1D..=0x1F => self.sprite_palettes[3].set_rgb(index - 0x1D, rgb(self.ram[index])),
            // TODO: Verify if any of these do anything. 0x10 may be a mirror of 0x00.
            0x04 | 0x08 | 0x0C | 0x10 | 0x14 | 0x18 | 0x1C => {}
            0x20.. => unreachable!(),
        }
    }

    pub fn to_slice(&self) -> &[MaskedByte<0b0011_1111>; PALETTE_RAM_SIZE] {
        &self.ram
    }

    #[inline]
    pub fn universal_background_rgb(&self) -> Rgb {
        self.universal_background_rgb
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
        let mut palette_ram = PaletteRam::new(SystemPalette::ALL_BLACK);
        assert_eq!(palette_ram.peek(&ppu_regs, 12).value(), 0b0000_0000);
        palette_ram.write(12, 0b1110_1010);
        assert_eq!(palette_ram.peek(&ppu_regs, 12).value(), 0b0010_1010);
    }
}
