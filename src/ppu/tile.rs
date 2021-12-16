use crate::ppu::palette::palette_index::PaletteIndex;
use crate::util::get_bit;

pub struct Tile<'a> {
    bytes: &'a [u8; 16],
}

impl <'a> Tile<'a> {
    pub fn new(bytes: &'a [u8; 16]) -> Tile<'a> {
        Tile {bytes}
    }

    #[inline]
    pub fn sliver_at(&self, row_in_tile: u8) -> [Option<PaletteIndex>; 8] {
        [
            self.palette_index_at(0, row_in_tile),
            self.palette_index_at(1, row_in_tile),
            self.palette_index_at(2, row_in_tile),
            self.palette_index_at(3, row_in_tile),
            self.palette_index_at(4, row_in_tile),
            self.palette_index_at(5, row_in_tile),
            self.palette_index_at(6, row_in_tile),
            self.palette_index_at(7, row_in_tile),
        ]
    }

    #[inline]
    pub fn palette_index_at(&self, column: u8, row: u8) -> Option<PaletteIndex> {
        let column = column as usize;
        let row = row as usize;
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
