use enum_iterator::IntoEnumIterator;

use crate::ppu::pixel_index::{PixelColumn, PixelRow, ColumnInTile, RowInTile};
use crate::ppu::palette::palette::Palette;
use crate::ppu::palette::palette_index::PaletteIndex;
use crate::ppu::palette::rgbt::Rgbt;
use crate::ppu::render::frame::Frame;
use crate::ppu::sprite::Sprite;
use crate::util::bit_util::get_bit;
use crate::util::mapped_array::MappedArray;

const PATTERN_TABLE_SIZE: usize = 0x1000;
const PATTERN_SIZE: usize = 16;

pub struct PatternTable<'a>(&'a MappedArray<4>);

impl <'a> PatternTable<'a> {
    pub fn new(raw: &MappedArray<4>) -> PatternTable {
        PatternTable(raw)
    }

    // Used for debug windows only.
    pub fn render_background_tile(
        &self,
        pattern_index: PatternIndex,
        palette: Palette,
        tile: &mut Tile,
    ) {
        for row_in_tile in RowInTile::into_enum_iter() {
            self.render_background_tile_sliver(
                pattern_index, row_in_tile, palette, &mut tile.0[row_in_tile as usize])
        }
    }

    #[inline]
    pub fn render_background_tile_sliver(
        &self,
        pattern_index: PatternIndex,
        row_in_background_tile: RowInTile,
        palette: Palette,
        tile_sliver: &mut [Rgbt; 8],
    ) {
        let index = PATTERN_SIZE * pattern_index.to_usize();
        let low_index = index + row_in_background_tile as usize;
        let high_index = low_index + 8;

        let low_byte = self.0.read(low_index);
        let high_byte = self.0.read(high_index);

        for (column_in_tile, pixel) in tile_sliver.iter_mut().enumerate() {
            let low_bit = get_bit(low_byte, column_in_tile);
            let high_bit = get_bit(high_byte, column_in_tile);
            *pixel =
                match (low_bit, high_bit) {
                    (false, false) => Rgbt::Transparent,
                    (true , false) => Rgbt::Opaque(palette[PaletteIndex::One]),
                    (false, true ) => Rgbt::Opaque(palette[PaletteIndex::Two]),
                    (true , true ) => Rgbt::Opaque(palette[PaletteIndex::Three]),
                };
        }
    }

    // No obvious way to reduce the number of parameters.
    #[allow(clippy::too_many_arguments)]
    #[inline]
    pub fn render_sprite_sliver(
        &self,
        sprite: Sprite,
        pattern_index: PatternIndex,
        palette: Palette,
        frame: &mut Frame,
        column: PixelColumn,
        row: PixelRow,
        row_in_sprite: RowInTile,
        is_sprite_0: bool,
    ) {
        let index = 16 * pattern_index.to_usize();
        let low_index = index + row_in_sprite as usize;
        let high_index = low_index + 8;

        let low_byte = self.0.read(low_index);
        let high_byte = self.0.read(high_index);

        let flip = sprite.flip_horizontally();
        for mut column_in_sprite in ColumnInTile::into_enum_iter() {
            let low_bit = get_bit(low_byte, column_in_sprite as usize);
            let high_bit = get_bit(high_byte, column_in_sprite as usize);
            let rgb =
                match (low_bit, high_bit) {
                    (false, false) => /* Transparent. */ continue,
                    (true , false) => palette[PaletteIndex::One],
                    (false, true ) => palette[PaletteIndex::Two],
                    (true , true ) => palette[PaletteIndex::Three],
                };

            if flip {
                column_in_sprite = column_in_sprite.flip();
            }

            if let Some(column) = column.add_column_in_tile(column_in_sprite) {
                frame.set_sprite_pixel(column, row, rgb, sprite.priority(), is_sprite_0);
            }
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum PatternTableSide {
    Left,
    Right,
}

impl PatternTableSide {
    pub fn from_index(index: usize) -> PatternTableSide {
        assert!(index < 2 * PATTERN_TABLE_SIZE);
        if index / PATTERN_TABLE_SIZE == 0 {
            PatternTableSide::Left
        } else {
            PatternTableSide::Right
        }
    }

    pub fn to_start_end(self) -> (usize, usize) {
        match self {
            PatternTableSide::Left  => (0x0000, PATTERN_TABLE_SIZE),
            PatternTableSide::Right => (PATTERN_TABLE_SIZE, 2 * PATTERN_TABLE_SIZE),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct PatternIndex(u8);

impl PatternIndex {
    pub fn new(value: u8) -> PatternIndex {
        PatternIndex(value)
    }

    pub fn to_tall_indexes(self) -> (PatternIndex, PatternIndex) {
        let first  = self.0 & 0b1111_1110;
        let second = self.0 | 0b0000_0001;
        (PatternIndex(first), PatternIndex(second))
    }

    pub fn to_u8(self) -> u8 {
        self.0
    }

    pub fn to_usize(self) -> usize {
        usize::from(self.0)
    }
}

pub struct Tile(pub [[Rgbt; 8]; 8]);

impl Tile {
    pub fn new() -> Tile {
        Tile([[Rgbt::Transparent; 8]; 8])
    }
}
