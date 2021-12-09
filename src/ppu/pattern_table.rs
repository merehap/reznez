use crate::ppu::palette::palette_index::PaletteIndex;
use crate::ppu::tile::Tile;

pub struct PatternTable<'a>(&'a [u8; 0x2000]);

impl <'a> PatternTable<'a> {
    pub fn new(raw: &'a [u8; 0x2000]) -> PatternTable<'a> {
        PatternTable(raw)
    }

    pub fn tile_at(&'a self, side: PatternTableSide, tile_index: u8) -> Tile<'a> {
        let mut start_index = match side {
            PatternTableSide::Left => 0x0,
            PatternTableSide::Right => 0x1000,
        };

        start_index += 16 * (tile_index as usize);

        Tile::new((&self.0[start_index..start_index + 16]).try_into().unwrap())
    }

    pub fn tile_sliver_at(
        &'a self,
        side: PatternTableSide,
        tile_index: u8,
        row_in_tile: u8,
        ) -> [Option<PaletteIndex>; 8] {

        self.tile_at(side, tile_index).sliver_at(row_in_tile)
    }
}

#[derive(Clone, Copy)]
pub enum PatternTableSide {
    Left,
    Right,
}
