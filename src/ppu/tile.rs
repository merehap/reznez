use crate::ppu::palette::palette_index::PaletteIndex;
use crate::util::get_bit;

pub struct Tile<'a> {
    bytes: &'a [u8; 16],
}

impl <'a> Tile<'a> {
    pub fn new(bytes: &'a [u8; 16]) -> Tile<'a> {
        Tile {bytes}
    }

    pub fn sliver_at(&self, row_in_tile: u8) -> [Option<PaletteIndex>; 8] {
        let sliver: Vec<_> = (0..8)
            .map(|column| self.palette_index_at(column, row_in_tile))
            .collect();
        sliver.try_into().unwrap()
    }

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
