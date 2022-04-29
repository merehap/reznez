use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

use crate::ppu::palette::palette_table::PaletteTable;
use crate::ppu::palette::palette_table_index::PaletteTableIndex;
use crate::ppu::palette::rgbt::Rgbt;
use crate::ppu::pattern_table::{PatternIndex, PatternTable, PatternTableSide};
use crate::ppu::pixel_index::{ColumnInTile, PixelColumn, PixelRow, RowInTile};
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
    #[rustfmt::skip]
    pub fn from_u32(value: u32) -> Sprite {
        let [y_coordinate, raw_pattern_index, attribute, x_coordinate] =
            value.to_be_bytes();

        let palette_table_index = match (get_bit(attribute, 6), get_bit(attribute, 7)) {
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

    pub fn priority(self) -> Priority {
        self.priority
    }

    pub fn render_sliver(
        self,
        row: PixelRow,
        sprite_height: SpriteHeight,
        pattern_table: &PatternTable,
        palette_table: &PaletteTable,
        is_sprite_0: bool,
        frame: &mut Frame,
    ) {
        let Some((sprite_half, row_in_half)) = self.row_in_sprite(sprite_height, row) else {
            return;
        };

        let mut sprite_sliver = [Rgbt::Transparent; 8];
        self.render_sliver_from_sprite_half(
            sprite_height,
            sprite_half,
            row_in_half,
            pattern_table,
            palette_table,
            &mut sprite_sliver,
        );

        for (column_in_sprite, &pixel) in sprite_sliver.iter().enumerate() {
            let column_in_sprite = ColumnInTile::from_usize(column_in_sprite).unwrap();
            if let Rgbt::Opaque(rgb) = pixel {
                if let Some(column) = self.x_coordinate.add_column_in_tile(column_in_sprite) {
                    frame.set_sprite_pixel(column, row, rgb, self.priority(), is_sprite_0);
                }
            }
        }
    }

    fn render_sliver_from_sprite_half(
        self,
        sprite_height: SpriteHeight,
        sprite_half: SpriteHalf,
        mut row_in_half: RowInTile,
        pattern_table: &PatternTable,
        palette_table: &PaletteTable,
        sprite_sliver: &mut [Rgbt; 8],
    ) {
        if self.flip_vertically {
            row_in_half = row_in_half.flip();
        }

        #[rustfmt::skip]
        let pattern_index = match (sprite_height, sprite_half) {
            (SpriteHeight::Normal, SpriteHalf::Upper) => self.pattern_index,
            (SpriteHeight::Normal, SpriteHalf::Lower) => unreachable!(),
            (SpriteHeight::Tall,   SpriteHalf::Upper) => self.pattern_index.to_tall_indexes().0,
            (SpriteHeight::Tall,   SpriteHalf::Lower) => self.pattern_index.to_tall_indexes().1,
        };

        let sprite_palette = palette_table.sprite_palette(self.palette_table_index);
        pattern_table.render_pixel_sliver(
            pattern_index,
            row_in_half,
            sprite_palette,
            sprite_sliver,
        );

        if self.flip_horizontally {
            for i in 0..sprite_sliver.len() / 2 {
                sprite_sliver.swap(i, 7 - i);
            }
        }
    }

    #[rustfmt::skip]
    fn row_in_sprite(
        self,
        sprite_height: SpriteHeight,
        pixel_row: PixelRow,
    ) -> Option<(SpriteHalf, RowInTile)> {
        let sprite_top_row = self.y_coordinate.to_pixel_row()?;
        let offset = pixel_row.difference(sprite_top_row)?;
        let row_in_half = FromPrimitive::from_u8(offset % 8).unwrap();
        match (offset / 8, sprite_height) {
            (0,                  _) => Some((SpriteHalf::Upper, row_in_half)),
            (1, SpriteHeight::Tall) => Some((SpriteHalf::Lower, row_in_half)),
            (_,                  _) => None,
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Priority {
    InFront,
    Behind,
}

#[derive(Clone, Copy, Debug)]
struct SpriteY(u8);

impl SpriteY {
    pub fn to_pixel_row(self) -> Option<PixelRow> {
        // Rendering of sprites is delayed by one scanline so the sprite ends
        // up rendered one scanline lower than would be expected.
        // Sprites with y >= 239 are valid but can't be rendered.
        // https://wiki.nesdev.org/w/index.php?title=PPU_OAM#Byte_0
        PixelRow::try_from_u8((u16::from(self.0) + 1).try_into().ok()?)
    }
}

#[derive(FromPrimitive)]
enum SpriteHalf {
    Upper,
    Lower,
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
