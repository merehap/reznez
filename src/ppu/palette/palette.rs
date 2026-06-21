use std::ops::Index;

use crate::ppu::palette::palette_index::PaletteIndex;
use crate::ppu::palette::color::Color;
use crate::ppu::palette::color_t::ColorT;

#[derive(Clone, Copy, Debug)]
pub struct Palette([Color; 3]);

impl Palette {
    pub const ALL_BLACK: Self = Self([Color::BLACK; 3]);

    pub fn new(raw: [Color; 3]) -> Palette {
        Palette(raw)
    }

    pub fn colors(self) -> [Color; 3] {
        self.0
    }

    pub fn color_t_from_low_high(self, low: bool, high: bool) -> ColorT {
        PaletteIndex::from_low_high(low, high)
            .map_or(ColorT::Transparent, |index| ColorT::Opaque(self.0[index as usize]))
    }

    pub fn color(self, index: usize) -> Color {
        self.0[index]
    }

    pub fn set_color(&mut self, index: usize, color: Color) {
        self.0[index] = color;
    }
}

impl Index<PaletteIndex> for Palette {
    type Output = Color;

    fn index(&self, palette_index: PaletteIndex) -> &Color {
        &self.0[palette_index as usize]
    }
}
