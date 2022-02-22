use enum_iterator::IntoEnumIterator;

use crate::ppu::pixel_index::{PixelColumn, PixelRow, RowInTile};
use crate::ppu::palette::palette_table::PaletteTable;
use crate::ppu::palette::palette_table_index::PaletteTableIndex;
use crate::ppu::pattern_table::{PatternTable, PatternIndex, PatternTableSide};
use crate::ppu::render::frame::Frame;
use crate::util::bit_util::get_bit;

#[derive(Clone, Copy, Debug)]
pub struct Sprite {
    x_coordinate: PixelColumn,
    // Sprites with invalid y_coordinates won't be rendered, but must be kept.
    y_coordinate: u8,
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
            y_coordinate,
            pattern_index: PatternIndex::new(raw_pattern_index),
            flip_vertically:   get_bit(attribute, 0),
            flip_horizontally: get_bit(attribute, 1),
            priority:          get_bit(attribute, 2).into(),
            palette_table_index,
        }
    }

    pub fn x_coordinate(self) -> PixelColumn {
        self.x_coordinate
    }

    pub fn y_coordinate(self) -> Option<PixelRow> {
        PixelRow::try_from_u8(self.y_coordinate)
    }

    #[inline]
    pub fn pattern_index(self) -> PatternIndex {
        self.pattern_index
    }

    #[inline]
    pub fn tall_sprite_pattern_table_side(self) -> PatternTableSide {
        if self.pattern_index.to_u8() & 1 == 0 {
            PatternTableSide::Left
        } else {
            PatternTableSide::Right
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

    pub fn is_in_bounds(self, x: PixelColumn, y: PixelRow) -> bool {
        x >= self.x_coordinate &&
            y.to_u8() >= self.y_coordinate &&
            x.to_usize() < self.x_coordinate.to_usize() + 8 &&
            y.to_usize() < self.y_coordinate as usize + 8
    }

    pub fn render_normal_height(
        self,
        pattern_table: &PatternTable,
        palette_table: &PaletteTable,
        is_sprite_0: bool,
        frame: &mut Frame,
    ) {
        self.render(pattern_table, self.pattern_index, palette_table, 0, is_sprite_0, frame);
    }

    pub fn render_tall(
        self,
        pattern_table: &PatternTable,
        palette_table: &PaletteTable,
        is_sprite_0: bool,
        frame: &mut Frame,
    ) {
        let (first_index, second_index) = self.pattern_index.to_tall_indexes();
        self.render(pattern_table, first_index, palette_table, 0, is_sprite_0, frame);
        self.render(pattern_table, second_index, palette_table, 8, is_sprite_0, frame);
    }

    fn render(
        self,
        pattern_table: &PatternTable,
        pattern_index: PatternIndex,
        palette_table: &PaletteTable,
        tall_sprite_offset: u8,
        is_sprite_0: bool,
        frame: &mut Frame,
    ) {
        let maybe_row = PixelRow::try_from_u8(self.y_coordinate)
            .map(|row| row.offset(tall_sprite_offset as i16))
            .flatten();
        let row;
        if let Some(r) = maybe_row {
            row = r;
        } else {
            return;
        }

        let column = self.x_coordinate;
        let sprite_palette = palette_table.sprite_palette(self.palette_table_index);
        for row_in_sprite in RowInTile::into_enum_iter() {
            let maybe_row =
                if self.flip_vertically {
                    row.add_flipped_row_in_tile(row_in_sprite)
                } else {
                    row.add_row_in_tile(row_in_sprite)
                };

            if let Some(row) = maybe_row {
                pattern_table.render_sprite_sliver(
                    self,
                    pattern_index,
                    sprite_palette,
                    frame,
                    column,
                    row,
                    row_in_sprite,
                    is_sprite_0,
                );
            } else {
                // FIXME: The part of vertically flipped sprites that is
                // on the screen should still be rendered.
                break;
            }
        }
    }
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
