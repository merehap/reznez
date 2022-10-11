use enum_iterator::IntoEnumIterator;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

use crate::ppu::palette::palette_table::PaletteTable;
use crate::ppu::palette::palette_table_index::PaletteTableIndex;
use crate::ppu::palette::rgbt::Rgbt;
use crate::ppu::pattern_table::{PatternIndex, PatternTable, PatternTableSide, Tile};
use crate::ppu::pixel_index::{ColumnInTile, PixelColumn, PixelRow, RowInTile};
use crate::ppu::register::registers::ctrl::SpriteHeight;
use crate::ppu::render::frame::Frame;
use crate::util::bit_util::get_bit;

#[derive(Clone, Copy, Debug)]
pub struct Sprite {
    x_coordinate: PixelColumn,
    y_coordinate: SpriteY,
    pattern_index: PatternIndex,
    attributes: SpriteAttributes,
}

impl Sprite {
    #[inline]
    #[rustfmt::skip]
    pub fn from_u32(value: u32) -> Sprite {
        let [y_coordinate, raw_pattern_index, attributes, x_coordinate] =
            value.to_be_bytes();

        Sprite {
            x_coordinate: PixelColumn::new(x_coordinate),
            y_coordinate: SpriteY(y_coordinate),
            pattern_index: PatternIndex::new(raw_pattern_index),
            attributes: SpriteAttributes::from_u8(attributes),
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
        self.attributes.priority
    }

    // For debug screens only.
    pub fn render_normal_height(
        self,
        pattern_table: &PatternTable,
        palette_table: &PaletteTable,
    ) -> Tile {
        self.render_tile(
            SpriteHeight::Normal,
            SpriteHalf::Top,
            pattern_table,
            palette_table,
        )
    }

    // For debug screens only.
    pub fn render_tall(
        self,
        pattern_table: &PatternTable,
        palette_table: &PaletteTable,
    ) -> (Tile, Tile) {
        use SpriteHalf::*;
        use SpriteHeight::Tall;
        (
            self.render_tile(Tall, Top, pattern_table, palette_table),
            self.render_tile(Tall, Bottom, pattern_table, palette_table),
        )
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
        let Some((sprite_half, row_in_half)) =
            Sprite::row_in_sprite(self.y_coordinate, self.attributes.flip_vertically, sprite_height, row) else {

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
            if let Rgbt::Opaque(_) = pixel {
                if let Some(column) =
                    self.x_coordinate.add_column_in_tile(column_in_sprite)
                {
                    frame.set_sprite_pixel(
                        column,
                        row,
                        pixel,
                        self.priority(),
                        is_sprite_0,
                    );
                }
            }
        }
    }

    fn render_tile(
        self,
        sprite_height: SpriteHeight,
        sprite_half: SpriteHalf,
        pattern_table: &PatternTable,
        palette_table: &PaletteTable,
    ) -> Tile {
        let mut tile = Tile::new();
        for row in RowInTile::into_enum_iter() {
            self.render_sliver_from_sprite_half(
                sprite_height,
                sprite_half,
                row,
                pattern_table,
                palette_table,
                tile.row_mut(row),
            );
        }

        tile
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
        #[rustfmt::skip]
        let pattern_index = match (sprite_height, sprite_half) {
            (SpriteHeight::Normal, SpriteHalf::Top) => self.pattern_index,
            (SpriteHeight::Normal, SpriteHalf::Bottom) => unreachable!(),
            (SpriteHeight::Tall,   SpriteHalf::Top) => self.pattern_index.to_tall_indexes().0,
            (SpriteHeight::Tall,   SpriteHalf::Bottom) => self.pattern_index.to_tall_indexes().1,
        };

        if self.attributes.flip_vertically {
            row_in_half = row_in_half.flip();
        }

        let sprite_palette = palette_table.sprite_palette(self.attributes.palette_table_index);
        pattern_table.render_pixel_sliver(
            pattern_index,
            row_in_half,
            sprite_palette,
            sprite_sliver,
        );

        if self.attributes.flip_horizontally {
            for i in 0..sprite_sliver.len() / 2 {
                sprite_sliver.swap(i, 7 - i);
            }
        }
    }

    #[rustfmt::skip]
    pub fn row_in_sprite(
        y_coordinate: SpriteY,
        flip_vertically: bool,
        sprite_height: SpriteHeight,
        pixel_row: PixelRow,
    ) -> Option<(SpriteHalf, RowInTile)> {
        let sprite_top_row = y_coordinate.to_pixel_row()?;
        let offset = pixel_row.difference(sprite_top_row)?;
        let row_in_half = FromPrimitive::from_u8(offset % 8).unwrap();
        match (offset / 8, sprite_height, flip_vertically) {
            (0, SpriteHeight::Normal, _    ) => Some((SpriteHalf::Top   , row_in_half)),
            (0, SpriteHeight::Tall  , false) => Some((SpriteHalf::Top   , row_in_half)),
            (0, SpriteHeight::Tall  , true ) => Some((SpriteHalf::Bottom, row_in_half)),
            (1, SpriteHeight::Tall  , false) => Some((SpriteHalf::Bottom, row_in_half)),
            (1, SpriteHeight::Tall  , true ) => Some((SpriteHalf::Top   , row_in_half)),
            (_, _                   , _    ) => None,
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Priority {
    InFront,
    Behind,
}

#[derive(Clone, Copy, Debug)]
pub struct SpriteY(u8);

impl SpriteY {
    pub fn new(value: u8) -> SpriteY {
        SpriteY(value)
    }

    pub fn to_pixel_row(self) -> Option<PixelRow> {
        // Rendering of sprites is delayed by one scanline so the sprite ends
        // up rendered one scanline lower than would be expected.
        // Sprites with y >= 239 are valid but can't be rendered.
        // https://wiki.nesdev.org/w/index.php?title=PPU_OAM#Byte_0
        PixelRow::try_from_u16(u16::from(self.0) + 1)
    }
}

#[derive(Clone, Copy, FromPrimitive)]
pub enum SpriteHalf {
    Top,
    Bottom,
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

#[derive(Clone, Copy, Debug)]
pub struct SpriteAttributes {
    flip_vertically: bool,
    flip_horizontally: bool,
    priority: Priority,
    palette_table_index: PaletteTableIndex,
}

impl SpriteAttributes {
    pub fn new() -> SpriteAttributes {
        SpriteAttributes {
            flip_vertically:   false,
            flip_horizontally: false,
            priority: Priority::InFront,
            palette_table_index: PaletteTableIndex::Zero,
        }
    }

    pub fn from_u8(value: u8) -> SpriteAttributes {
        let palette_table_index = match (get_bit(value, 6), get_bit(value, 7)) {
            (false, false) => PaletteTableIndex::Zero,
            (false, true ) => PaletteTableIndex::One,
            (true , false) => PaletteTableIndex::Two,
            (true , true ) => PaletteTableIndex::Three,
        };

        SpriteAttributes {
            flip_vertically:   get_bit(value, 0),
            flip_horizontally: get_bit(value, 1),
            priority:          get_bit(value, 2).into(),
            palette_table_index,
        }
    }

    pub fn flip_vertically(self) -> bool {
        self.flip_vertically
    }

    pub fn flip_horizontally(self) -> bool {
        self.flip_horizontally
    }

    pub fn priority(self) -> Priority {
        self.priority
    }

    pub fn palette_table_index(self) -> PaletteTableIndex {
        self.palette_table_index
    }
}
