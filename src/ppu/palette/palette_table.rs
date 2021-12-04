use crate::ppu::palette::color::Color;
use crate::ppu::palette::palette::Palette;
use crate::ppu::palette::palette_table_index::PaletteTableIndex;

pub struct PaletteTable<'a>(&'a [u8; 0x20]);

impl <'a> PaletteTable<'a> {
    pub fn new(raw: &'a [u8; 0x20]) -> PaletteTable<'a> {
        PaletteTable(raw)
    }

    pub fn universal_background_color(&self) -> Color {
        Color::from_u8(self.0[0]).unwrap()
    }

    pub fn background_palette(&self, number: PaletteTableIndex) -> Palette {
        let start = 4 * (number as usize) + 1;
        Palette::new((&self.0[start..start + 3]).try_into().unwrap())
    }

    pub fn sprite_palette(&self, number: PaletteTableIndex) -> Palette {
        let start = 4 * (number as usize) + 0x11;
        Palette::new((&self.0[start..start + 3]).try_into().unwrap())
    }
}
