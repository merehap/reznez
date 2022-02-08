use crate::ppu::palette::palette_table::PaletteTable;
use crate::ppu::palette::palette_table_index::PaletteTableIndex;
use crate::ppu::pattern_table::{PatternTable, PatternIndex, PatternTableSide};
use crate::ppu::render::frame::Frame;
use crate::util::bit_util::get_bit;

#[derive(Clone, Copy, Debug)]
pub struct Sprite {
    x_coordinate: u8,
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
            x_coordinate,
            y_coordinate,
            pattern_index: PatternIndex::new(raw_pattern_index),
            flip_vertically:   get_bit(attribute, 0),
            flip_horizontally: get_bit(attribute, 1),
            priority:          get_bit(attribute, 2).into(),
            palette_table_index,
        }
    }

    pub fn x_coordinate(self) -> u8 {
        self.x_coordinate
    }

    pub fn y_coordinate(self) -> u8 {
        self.y_coordinate
    }

    #[inline]
    pub fn pattern_index(self) -> PatternIndex {
        self.pattern_index
    }

    #[inline]
    pub fn tall_sprite_info(self) -> (PatternIndex, PatternTableSide) {
        let tall_pattern_index = self.pattern_index.into_wide_index();
        let tall_pattern_table_side =
            if self.pattern_index.to_u8() & 1 == 0 {
                PatternTableSide::Left
            } else {
                PatternTableSide::Right
            };
        (tall_pattern_index, tall_pattern_table_side)
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

    pub fn render(
        self,
        pattern_table: &PatternTable,
        palette_table: &PaletteTable,
        is_sprite0: bool,
        frame: &mut Frame,
    ) {
        let column = self.x_coordinate();
        let row = self.y_coordinate();
        let sprite_palette = palette_table.sprite_palette(self.palette_table_index());

        for row_in_sprite in 0..8 {
            let row =
                if self.flip_vertically {
                    row + 7 - row_in_sprite
                } else {
                    row + row_in_sprite
                };

            if row >= 240 {
                // FIXME: The part of vertically flipped sprites that is
                // off the screen should still be rendered.
                break;
            }

            pattern_table.render_sprite_sliver(
                self,
                self.pattern_index,
                is_sprite0,
                sprite_palette,
                frame,
                column,
                row,
                row_in_sprite as usize,
            );
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
