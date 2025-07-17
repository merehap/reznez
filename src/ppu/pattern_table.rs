// modular_bitfield pedantic clippy warnings
#![allow(clippy::cast_lossless, clippy::no_effect_underscore_binding, clippy::map_unwrap_or)]

use enum_iterator::all;
use modular_bitfield::Specifier;

use crate::memory::raw_memory::RawMemorySlice;
use crate::ppu::palette::palette::Palette;
use crate::ppu::palette::rgbt::Rgbt;
use crate::ppu::pixel_index::{PixelRow, ColumnInTile, RowInTile};
use crate::ppu::sprite::sprite_half::SpriteHalf;
use crate::ppu::sprite::sprite_height::SpriteHeight;
use crate::ppu::sprite::sprite_y::SpriteY;
use crate::util::bit_util::get_bit;
use crate::util::unit::KIBIBYTE;

const PATTERN_TABLE_SIZE: u32 = 4 * KIBIBYTE;
const PATTERN_SIZE: u32 = 16;

// Used for debug window purposes only. The actual rendering pipeline deals with unabstracted bytes.
pub struct PatternTable<'a>([RawMemorySlice<'a>; 4]);

impl<'a> PatternTable<'a> {
    pub fn new(raw: [RawMemorySlice<'a>; 4]) -> PatternTable<'a> {
        PatternTable(raw)
    }

    pub fn render_pixel(
        &self,
        tile_number: TileNumber,
        column_in_tile: ColumnInTile,
        row_in_tile: RowInTile,
        palette: Palette,
        pixel: &mut Rgbt,
    ) {
        let index = PATTERN_SIZE * u32::from(tile_number);
        let low_index = index + row_in_tile as u32;
        let high_index = low_index + 8;

        let low_byte = self.read(low_index);
        let high_byte = self.read(high_index);

        let low_bit = get_bit(low_byte, column_in_tile as u32);
        let high_bit = get_bit(high_byte, column_in_tile as u32);
        *pixel = palette.rgbt_from_low_high(low_bit, high_bit);
    }

    fn read(&self, index: u32) -> u8 {
        let quadrant = index / KIBIBYTE;
        assert!(quadrant < 5);

        let offset = index % KIBIBYTE;

        self.0[quadrant as usize][offset]
    }

    pub fn render_background_tile(
        &self,
        tile_number: TileNumber,
        palette: Palette,
        tile: &mut Tile,
    ) {
        for row_in_tile in all::<RowInTile>() {
            self.render_pixel_sliver(
                tile_number,
                row_in_tile,
                palette,
                &mut tile.0[row_in_tile as usize],
            );
        }
    }

    #[inline]
    #[rustfmt::skip]
    pub fn render_pixel_sliver(
        &self,
        tile_number: TileNumber,
        row_in_tile: RowInTile,
        palette: Palette,
        tile_sliver: &mut [Rgbt; 8],
    ) {
        let index = PATTERN_SIZE * u32::from(tile_number);
        let low_index = index + row_in_tile as u32;
        let high_index = low_index + 8;

        let low_byte = self.read(low_index);
        let high_byte = self.read(high_index);

        for (column_in_tile, pixel) in tile_sliver.iter_mut().enumerate() {
            let low_bit = get_bit(low_byte, column_in_tile as u32);
            let high_bit = get_bit(high_byte, column_in_tile as u32);
            *pixel = palette.rgbt_from_low_high(low_bit, high_bit);
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug, Specifier)]
pub enum PatternTableSide {
    Left,
    Right,
}

impl PatternTableSide {
    pub fn from_index(index: u32) -> PatternTableSide {
        assert!(index < 2 * PATTERN_TABLE_SIZE);
        if index / PATTERN_TABLE_SIZE == 0 {
            PatternTableSide::Left
        } else {
            PatternTableSide::Right
        }
    }

    pub fn to_start_end(self) -> (u32, u32) {
        match self {
            PatternTableSide::Left => (0x0000, PATTERN_TABLE_SIZE),
            PatternTableSide::Right => (PATTERN_TABLE_SIZE, 2 * PATTERN_TABLE_SIZE),
        }
    }
}

impl From<bool> for PatternTableSide {
    fn from(value: bool) -> PatternTableSide {
        if value {
            PatternTableSide::Right
        } else {
            PatternTableSide::Left
        }
    }
}

impl From<PatternTableSide> for u16 {
    fn from(value: PatternTableSide) -> Self {
        value as u16
    }
}

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

pub struct Tile(pub [[Rgbt; 8]; 8]);

impl Tile {
    pub fn new() -> Tile {
        Tile([[Rgbt::Transparent; 8]; 8])
    }

    pub fn row_mut(&mut self, row: RowInTile) -> &mut [Rgbt; 8] {
        &mut self.0[row as usize]
    }
}
