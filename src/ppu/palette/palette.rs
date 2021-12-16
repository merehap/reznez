use std::ops::Index;

use crate::ppu::palette::color::Color;
use crate::ppu::palette::palette_index::PaletteIndex;
use crate::ppu::palette::rgb::Rgb;

#[derive(Clone, Copy)]
pub struct Palette([Rgb; 3]);

impl Palette {
    pub fn new(raw: [Rgb; 3]) -> Palette {
        Palette(raw)
    }
}

impl Index<PaletteIndex> for Palette {
    type Output = Rgb;

    fn index(&self, palette_index: PaletteIndex) -> &Rgb {
        &self.0[palette_index as usize]
    }
}
