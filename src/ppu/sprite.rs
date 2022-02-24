use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

use crate::ppu::pixel_index::{PixelColumn, PixelRow, RowInTile};
use crate::ppu::palette::palette_table::PaletteTable;
use crate::ppu::palette::palette_table_index::PaletteTableIndex;
use crate::ppu::pattern_table::{PatternTable, PatternIndex, PatternTableSide};
use crate::ppu::register::registers::ctrl::SpriteHeight;
use crate::ppu::render::frame::Frame;
use crate::util::bit_util::get_bit;

#[derive(Clone, Copy, Debug)]
pub struct Sprite {
    x_coordinate: PixelColumn,
    y_coordinate: SpriteY,
    pattern_index: PatternIndex,
    flip_vertically: bool,
    flip_horizontally: bool,
    priority: Priority,
    palette_table_index: PaletteTableIndex,
}

impl Sprite {
    #[inline]
    pub fn from_u32(value: u32) -> Sprite {
        let [y_coordinate, raw_pattern_index, attribute, x_coordinate] =
            value.to_be_bytes();

        let palette_table_index =
            match (get_bit(attribute, 6), get_bit(attribute, 7)) {
                (false, false) => PaletteTableIndex::Zero,
                (false, true ) => PaletteTableIndex::One,
                (true , false) => PaletteTableIndex::Two,
                (true , true ) => PaletteTableIndex::Three,
            };

        Sprite {
            x_coordinate: PixelColumn::new(x_coordinate),
            y_coordinate: SpriteY(y_coordinate),
            pattern_index: PatternIndex::new(raw_pattern_index),
            flip_vertically:   get_bit(attribute, 0),
            flip_horizontally: get_bit(attribute, 1),
            priority:          get_bit(attribute, 2).into(),
            palette_table_index,
        }
    }

    #[inline]
    pub fn tall_sprite_pattern_table_side(self) -> PatternTableSide {
        if self.pattern_index.to_u8() & 1 == 0 {
            PatternTableSide::Left
        } else {
            PatternTableSide::Right
        }
    }

    pub fn flip_horizontally(self) -> bool {
        self.flip_horizontally
    }

    pub fn priority(self) -> Priority {
        self.priority
    }

    pub fn row_in_sprite(
        self,
        sprite_height: SpriteHeight,
        pixel_row: PixelRow,
    ) -> Option<(SpriteHalf, RowInTile)> {

        if let Some(sprite_top_row) = self.y_coordinate.to_pixel_row() {
            if let Some(offset) = pixel_row.difference(sprite_top_row) {
                let row_in_sprite = FromPrimitive::from_u8(offset % 8).unwrap();
                let result =
                    match (offset / 8, sprite_height) {
                        (0,                  _) => Some((SpriteHalf::Upper, row_in_sprite)),
                        (1, SpriteHeight::Tall) => Some((SpriteHalf::Lower, row_in_sprite)),
                        (_,                  _) => None,
                    };
                return result;
            }
        }

        None
    }

    pub fn render_sliver(
        self,
        pixel_row: PixelRow,
        sprite_height: SpriteHeight,
        pattern_table: &PatternTable,
        palette_table: &PaletteTable,
        is_sprite_0: bool,
        frame: &mut Frame,
    ) {
        let sprite_palette = palette_table.sprite_palette(self.palette_table_index);

        if let Some((sprite_half, mut row_in_sprite)) =
            self.row_in_sprite(sprite_height, pixel_row) {

            if self.flip_vertically {
                row_in_sprite = row_in_sprite.flip();
            }

            let pattern_index =
                match (sprite_height, sprite_half) {
                    (SpriteHeight::Normal, SpriteHalf::Upper) => self.pattern_index,
                    (SpriteHeight::Normal, SpriteHalf::Lower) => unreachable!(),
                    (SpriteHeight::Tall  , SpriteHalf::Upper) => self.pattern_index.to_tall_indexes().0,
                    (SpriteHeight::Tall  , SpriteHalf::Lower) => self.pattern_index.to_tall_indexes().1,
                };
            pattern_table.render_sprite_sliver(
                self,
                pattern_index,
                sprite_palette,
                frame,
                self.x_coordinate,
                pixel_row,
                row_in_sprite,
                is_sprite_0,
            );
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct SpriteY(u8);

impl SpriteY {
    pub fn to_pixel_row(self) -> Option<PixelRow> {
        // Rendering of sprites is delayed by one scanline so the sprite ends
        // up rendered one scanline lower than would be expected.
        // Sprites with y >= 239 are valid but can't be rendered.
        // https://wiki.nesdev.org/w/index.php?title=PPU_OAM#Byte_0
        PixelRow::try_from_u8((self.0 as u16 + 1).try_into().ok()?)
    }
}

#[derive(FromPrimitive)]
pub enum SpriteHalf {
    Upper,
    Lower,
}

#[derive(Clone, Copy, Debug)]
pub enum Priority {
    InFront,
    Behind,
}

impl From<bool> for Priority {
    fn from(value: bool) -> Self {
        if value {
            Priority::Behind
        } else {
            Priority::InFront
        }
    }
}
