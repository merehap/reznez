use itertools::structs::Product;
use itertools::Itertools;
use num_traits::FromPrimitive;

use crate::ppu::pixel_index::{ColumnInTile, PixelColumn, PixelRow, RowInTile};

#[derive(Clone, Copy, Debug)]
pub struct BackgroundTileIndex {
    column: TileColumn,
    row: TileRow,
}

impl BackgroundTileIndex {
    pub fn iter() -> BackgroundTileIndexIterator {
        BackgroundTileIndexIterator(TileRow::iter().cartesian_product(TileColumn::iter()))
    }

    pub fn from_tile_column_row(column: TileColumn, row: TileRow) -> BackgroundTileIndex {
        BackgroundTileIndex { column, row }
    }

    pub fn to_usize(self) -> usize {
        TileColumn::COLUMN_COUNT * self.row.to_usize() + self.column.to_usize()
    }

    #[inline]
    pub fn tile_column(self) -> TileColumn {
        self.column
    }

    #[inline]
    pub fn tile_row(self) -> TileRow {
        self.row
    }
}

pub struct BackgroundTileIndexIterator(Product<TileRowIterator, TileColumnIterator>);

impl Iterator for BackgroundTileIndexIterator {
    type Item = BackgroundTileIndex;

    fn next(&mut self) -> Option<BackgroundTileIndex> {
        self.0
            .next()
            .map(|(row, column)| BackgroundTileIndex { column, row })
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub struct TileColumn(u8);

impl TileColumn {
    pub const ZERO: TileColumn = TileColumn(0);
    const MAX: TileColumn = TileColumn(31);
    const COLUMN_COUNT: usize = 32;

    pub fn iter() -> TileColumnIterator {
        TileColumnIterator(0)
    }

    pub fn increment(&mut self) -> bool {
        let will_wrap = *self == TileColumn::MAX;
        if will_wrap {
            self.0 = 0;
        } else {
            self.0 += 1;
        }

        will_wrap
    }

    pub fn to_pixel_column(self, column_in_tile: ColumnInTile) -> PixelColumn {
        // Unwrap always succeeds since 248 + 7 == 255.
        self.pixel_column()
            .add_column_in_tile(column_in_tile)
            .unwrap()
    }

    pub fn to_u8(self) -> u8 {
        self.0
    }

    pub fn to_u16(self) -> u16 {
        u16::from(self.0)
    }

    pub fn to_usize(self) -> usize {
        usize::from(self.0)
    }

    pub fn is_max(self) -> bool {
        self == TileColumn::MAX
    }

    pub fn try_from_u8(tile_column: u8) -> Option<TileColumn> {
        if usize::from(tile_column) < TileColumn::COLUMN_COUNT {
            Some(TileColumn(tile_column))
        } else {
            None
        }
    }

    fn pixel_column(self) -> PixelColumn {
        PixelColumn::new(8 * self.0)
    }
}

#[derive(Clone)]
pub struct TileColumnIterator(u8);

impl Iterator for TileColumnIterator {
    type Item = TileColumn;

    fn next(&mut self) -> Option<TileColumn> {
        let result = TileColumn::try_from_u8(self.0);
        self.0 += 1;
        result
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub struct TileRow(u8);

impl TileRow {
    pub const ZERO: TileRow = TileRow(0);
    const ROW_COUNT: u8 = 32;
    const MAX: TileRow = TileRow(31);
    const MAX_VISIBLE: TileRow = TileRow(29);

    pub fn iter() -> TileRowIterator {
        TileRowIterator(0)
    }

    pub fn increment(&mut self) -> bool {
        let should_wrap = *self == TileRow::MAX;
        if should_wrap {
            self.0 = 0;
        } else {
            self.0 += 1;
        }

        should_wrap
    }

    pub fn increment_visible(&mut self) -> bool {
        let should_wrap = *self == TileRow::MAX_VISIBLE;
        if *self == TileRow::MAX_VISIBLE || *self == TileRow::MAX {
            self.0 = 0;
        } else {
            self.0 += 1;
        }

        should_wrap
    }

    pub fn from_pixel_row(pixel_row: PixelRow) -> (TileRow, RowInTile) {
        let tile_row = TileRow(pixel_row.to_u8() / 8);
        let row_in_tile = FromPrimitive::from_u8(pixel_row.to_u8() % 8).unwrap();
        (tile_row, row_in_tile)
    }

    pub fn to_u8(self) -> u8 {
        self.0
    }

    pub fn to_u16(self) -> u16 {
        u16::from(self.0)
    }

    pub fn to_usize(self) -> usize {
        usize::from(self.0)
    }

    pub const fn try_from_u8(tile_row: u8) -> Option<TileRow> {
        if tile_row < TileRow::ROW_COUNT {
            Some(TileRow(tile_row))
        } else {
            None
        }
    }
}

#[derive(Clone)]
pub struct TileRowIterator(u8);

impl Iterator for TileRowIterator {
    type Item = TileRow;

    fn next(&mut self) -> Option<TileRow> {
        let result = TileRow::try_from_u8(self.0);
        self.0 += 1;
        result
    }
}
