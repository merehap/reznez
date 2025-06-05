use crate::ppu::palette::color::Color;
use crate::ppu::palette::palette::Palette;
use crate::ppu::palette::palette_table_index::PaletteTableIndex;
use crate::ppu::palette::rgb::Rgb;
use crate::ppu::palette::system_palette::SystemPalette;
use crate::ppu::register::registers::mask::Mask;

// TODO: Support https://wiki.nesdev.org/w/index.php?title=PPU_palettes#The_background_palette_hack
#[derive(Debug)]
pub struct PaletteTable {
    universal_background_rgb: Rgb,
    background_palettes: [Palette; 4],
    sprite_palettes: [Palette; 4],
}

impl PaletteTable {
    // FIXME: This is called once per frame, but there should be a way to cache the lookup_rgb calls.
    pub fn new(raw: &[u8; 0x20], system_palette: &SystemPalette, mask: Mask) -> PaletteTable {
        let rgb = |raw_color: u8| -> Rgb {
            system_palette.lookup_rgb(Color::from_u8(raw_color), mask)
        };

        let background_palettes = [
            Palette::new([rgb(raw[0x01]), rgb(raw[0x02]), rgb(raw[0x03])]),
            Palette::new([rgb(raw[0x05]), rgb(raw[0x06]), rgb(raw[0x07])]),
            Palette::new([rgb(raw[0x09]), rgb(raw[0x0A]), rgb(raw[0x0B])]),
            Palette::new([rgb(raw[0x0D]), rgb(raw[0x0E]), rgb(raw[0x0F])]),
        ];
        let sprite_palettes = [
            Palette::new([rgb(raw[0x11]), rgb(raw[0x12]), rgb(raw[0x13])]),
            Palette::new([rgb(raw[0x15]), rgb(raw[0x16]), rgb(raw[0x17])]),
            Palette::new([rgb(raw[0x19]), rgb(raw[0x1A]), rgb(raw[0x1B])]),
            Palette::new([rgb(raw[0x1D]), rgb(raw[0x1E]), rgb(raw[0x1F])]),
        ];

        PaletteTable {
            universal_background_rgb: rgb(raw[0x00]),
            background_palettes,
            sprite_palettes,
        }
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

    pub fn background_palettes(&self) -> [Palette; 4] {
        self.background_palettes
    }

    pub fn sprite_palettes(&self) -> [Palette; 4] {
        self.sprite_palettes
    }
}
