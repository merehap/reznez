use std::ops::Index;

use crate::ppu::palette::palette_index::PaletteIndex;
use crate::ppu::palette::rgb::Rgb;
use crate::ppu::palette::rgbt::Rgbt;

#[derive(Clone, Copy, Debug)]
pub struct Palette([Rgb; 3]);

impl Palette {
    pub fn new(raw: [Rgb; 3]) -> Palette {
        Palette(raw)
    }

    pub fn rgbs(self) -> [Rgb; 3] {
        self.0
    }

    pub fn rgbt_from_low_high(self, low: bool, high: bool) -> Rgbt {
        PaletteIndex::from_low_high(low, high)
            .map_or(Rgbt::Transparent, |index| Rgbt::Opaque(self.0[index as usize]))
    }
}

impl Index<PaletteIndex> for Palette {
    type Output = Rgb;

    fn index(&self, palette_index: PaletteIndex) -> &Rgb {
        &self.0[palette_index as usize]
    }
}
