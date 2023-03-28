use num_traits::FromPrimitive;

use crate::ppu::pixel_index::{PixelRow, RowInTile};
use crate::ppu::sprite::sprite_half::SpriteHalf;
use crate::ppu::sprite::sprite_height::SpriteHeight;

#[derive(Clone, Copy, Debug)]
pub struct SpriteY(u8);

impl SpriteY {
    pub fn new(value: u8) -> SpriteY {
        SpriteY(value)
    }

    pub fn to_current_pixel_row(self) -> Option<PixelRow> {
        PixelRow::try_from_u16(u16::from(self.0))
    }

    pub fn row_in_sprite(
        self,
        flip_vertically: bool,
        sprite_height: SpriteHeight,
        pixel_row: PixelRow,
    ) -> Option<(SpriteHalf, RowInTile, bool)> {
        let visible = self.to_current_pixel_row().is_some();
        let y_offset = pixel_row.to_u8().checked_sub(self.0)?;

        let mut row_in_half: RowInTile = FromPrimitive::from_u8(y_offset % 8).unwrap();
        let mut sprite_half = sprite_height.sprite_half(y_offset)?;
        if flip_vertically {
            row_in_half = row_in_half.flip();
            if sprite_height == SpriteHeight::Tall {
                sprite_half = sprite_half.flip();
            }
        }

        Some((sprite_half, row_in_half, visible))
    }
}
