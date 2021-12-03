use crate::ppu::palette::palette_index::PaletteIndex;
use crate::util::get_bit;

pub struct Tile<'a> {
    bytes: &'a [u8; 16],
}

impl <'a> Tile<'a> {
    pub fn new(bytes: &'a [u8; 16]) -> Tile<'a> {
        Tile {bytes}
    }

    pub fn palette_index_at(&self, column: usize, row: usize) -> Option<PaletteIndex> {
        let low_bit = get_bit(self.bytes[row], column);
        let high_bit = get_bit(self.bytes[row + 8], column);
        match (low_bit, high_bit) {
            (false, false) => None,
            (true , false) => Some(PaletteIndex::One),
            (false, true ) => Some(PaletteIndex::Two),
            (true , true ) => Some(PaletteIndex::Three),
        }
    }
}
