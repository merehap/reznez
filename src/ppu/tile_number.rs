use crate::ppu::pattern_table_side::PatternTableSide;
use crate::ppu::pixel_index::{PixelRow, RowInTile};
use crate::ppu::sprite::sprite_half::SpriteHalf;
use crate::ppu::sprite::sprite_height::SpriteHeight;
use crate::ppu::sprite::sprite_y::SpriteY;

#[derive(Clone, Copy, Debug)]
pub struct TileNumber(u8);

impl TileNumber {
    pub fn new(value: u8) -> TileNumber {
        TileNumber(value)
    }

    pub fn number_and_row(
        self,
        sprite_top_row: SpriteY,
        flip_vertically: bool,
        sprite_height: SpriteHeight,
        pixel_row: PixelRow,
    ) -> Option<(TileNumber, RowInTile, bool)> {
        let (sprite_half, row_in_half, visible) =
            sprite_top_row.row_in_sprite(flip_vertically, sprite_height, pixel_row)?;

        #[rustfmt::skip]
        let tile_number = match (sprite_height, sprite_half) {
            (SpriteHeight::Normal, SpriteHalf::Top   ) => self,
            (SpriteHeight::Normal, SpriteHalf::Bottom) => unreachable!(),
            (SpriteHeight::Tall  , SpriteHalf::Top   ) => self.to_tall_tile_numbers().0,
            (SpriteHeight::Tall  , SpriteHalf::Bottom) => self.to_tall_tile_numbers().1,
        };

        Some((tile_number, row_in_half, visible))
    }

    pub fn to_tall_tile_numbers(self) -> (TileNumber, TileNumber) {
        let first  = self.0 & 0b1111_1110;
        let second = self.0 | 0b0000_0001;
        (TileNumber(first), TileNumber(second))
    }

    #[inline]
    pub fn tall_sprite_pattern_table_side(self) -> PatternTableSide {
        if self.0 & 1 == 0 {
            PatternTableSide::Left
        } else {
            PatternTableSide::Right
        }
    }
}

impl From<TileNumber> for u16 {
    fn from(value: TileNumber) -> Self {
        value.0.into()
    }
}

impl From<TileNumber> for u32 {
    fn from(value: TileNumber) -> Self {
        value.0.into()
    }
}

impl From<TileNumber> for usize {
    fn from(value: TileNumber) -> Self {
        value.0.into()
    }
}