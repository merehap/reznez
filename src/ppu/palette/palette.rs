use std::ops::Index;

use crate::ppu::palette::color::Color;
use crate::ppu::palette::palette_index::PaletteIndex;

pub struct Palette([Color; 3]);

impl Palette {
    pub fn new(raw: [u8; 3]) -> Palette {
        Palette(raw.map(Color::new))
    }
}

impl Index<PaletteIndex> for Palette {
    type Output = Color;

    fn index(&self, palette_index: PaletteIndex) -> &Color {
        &self.0[palette_index as usize]
    }
}
