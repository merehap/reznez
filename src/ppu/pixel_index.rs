use enum_iterator::IntoEnumIterator;
use itertools::structs::Product;
use itertools::Itertools;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

use crate::ppu::clock::Clock;

#[derive(Clone, Copy)]
pub struct PixelIndex {
    column: PixelColumn,
    row: PixelRow,
}

impl PixelIndex {
    pub const PIXEL_COUNT: usize = PixelColumn::COLUMN_COUNT * PixelRow::ROW_COUNT;

    pub fn iter() -> PixelIndexIterator {
        PixelIndexIterator::new()
    }

    pub fn try_from_clock(clock: &Clock) -> Option<PixelIndex> {
        PixelIndex::try_from_scanline_cycle(clock.scanline(), clock.cycle())
    }

    pub fn to_column_row(self) -> (PixelColumn, PixelRow) {
        (self.column, self.row)
    }

    pub fn to_usize(self) -> usize {
        PixelColumn::COLUMN_COUNT * self.row.to_usize() + self.column.to_usize()
    }

    fn try_from_scanline_cycle(scanline: u16, cycle: u16) -> Option<PixelIndex> {
        let column = PixelColumn::try_from_u16(cycle - 1)?;
        let row = PixelRow::try_from_u16(scanline)?;
        Some(PixelIndex { column, row })
    }
}

pub struct PixelIndexIterator(Product<PixelRowIterator, PixelColumnIterator>);

impl PixelIndexIterator {
    pub fn new() -> PixelIndexIterator {
        PixelIndexIterator(PixelRow::iter().cartesian_product(PixelColumn::iter()))
    }
}

impl Iterator for PixelIndexIterator {
    type Item = PixelIndex;

    fn next(&mut self) -> Option<PixelIndex> {
        self.0
            .next()
            .map(|(row, column)| PixelIndex { column, row })
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub struct PixelColumn(u8);

impl PixelColumn {
    pub const COLUMN_COUNT: usize = 256;
    pub const MAX: PixelColumn = PixelColumn::new(255);

    pub fn iter() -> PixelColumnIterator {
        PixelColumnIterator(0)
    }

    pub const fn new(pixel_column: u8) -> PixelColumn {
        PixelColumn(pixel_column)
    }

    pub fn try_from_u16(pixel_column: u16) -> Option<PixelColumn> {
        Some(PixelColumn::new(pixel_column.try_into().ok()?))
    }

    pub fn add_column_in_tile(self, column_in_tile: ColumnInTile) -> Option<PixelColumn> {
        let value = self.0.checked_add(column_in_tile as u8)?;
        Some(PixelColumn::new(value))
    }

    pub fn offset(self, offset: i16) -> Option<PixelColumn> {
        let column: i16 = i16::from(self.0) + offset;
        if column < 0 {
            return None;
        }

        column.try_into().ok().map(PixelColumn)
    }

    pub fn is_in_left_margin(self) -> bool {
        self.0 < 8
    }

    pub fn to_u8(self) -> u8 {
        self.0
    }

    pub fn to_usize(self) -> usize {
        self.0 as usize
    }
}

#[derive(Clone, Copy)]
pub struct PixelColumnIterator(u16);

impl Iterator for PixelColumnIterator {
    type Item = PixelColumn;

    fn next(&mut self) -> Option<PixelColumn> {
        let result = PixelColumn::try_from_u16(self.0);
        self.0 += 1;
        result
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub struct PixelRow(u8);

impl PixelRow {
    pub const ROW_COUNT: usize = 240;
    pub const MAX: PixelRow = PixelRow(PixelRow::ROW_COUNT as u8 - 1);

    pub fn iter() -> PixelRowIterator {
        PixelRowIterator(0)
    }

    pub const fn try_from_u8(pixel_row: u8) -> Option<PixelRow> {
        if (pixel_row as usize) < PixelRow::ROW_COUNT {
            Some(PixelRow(pixel_row))
        } else {
            None
        }
    }

    pub fn try_from_u16(pixel_row: u16) -> Option<PixelRow> {
        PixelRow::try_from_u8(pixel_row.try_into().ok()?)
    }

    pub const fn saturate_from_u8(pixel_row: u8) -> PixelRow {
        PixelRow::try_from_u8(pixel_row).unwrap_or(PixelRow::MAX)
    }

    pub fn add_row_in_tile(self, row_in_tile: RowInTile) -> Option<PixelRow> {
        let value = self.0.checked_add(row_in_tile as u8)?;
        PixelRow::try_from_u8(value)
    }

    pub fn offset(self, offset: i16) -> Option<PixelRow> {
        let row: i16 = i16::from(self.0) + offset;
        if (0..240).contains(&row) {
            Some(PixelRow::try_from_u8(row as u8).unwrap())
        } else {
            None
        }
    }

    pub fn difference(self, other: PixelRow) -> Option<u8> {
        self.to_u8().checked_sub(other.to_u8())
    }

    pub fn to_u8(self) -> u8 {
        self.0
    }

    pub fn to_usize(self) -> usize {
        usize::from(self.0)
    }
}

#[derive(Clone, Copy)]
pub struct PixelRowIterator(u8);

impl Iterator for PixelRowIterator {
    type Item = PixelRow;

    fn next(&mut self) -> Option<PixelRow> {
        let result = PixelRow::try_from_u8(self.0);
        self.0 += 1;
        result
    }
}

#[derive(
    PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug, FromPrimitive, IntoEnumIterator,
)]
pub enum ColumnInTile {
    Zero,
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
}

impl ColumnInTile {
    pub fn flip(self) -> ColumnInTile {
        FromPrimitive::from_u8(7 - (self as u8)).unwrap()
    }
}

#[derive(
    PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug, FromPrimitive, IntoEnumIterator,
)]
pub enum RowInTile {
    Zero,
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
}

impl RowInTile {
    pub fn flip(self) -> RowInTile {
        FromPrimitive::from_u8(7 - (self as u8)).unwrap()
    }

    pub fn increment_low_bits(&mut self) {
        self.increment();
        *self = RowInTile::from_u8((*self as u8) & 0b11).unwrap();
    }

    pub fn increment(&mut self) -> bool {
        let will_wrap = *self == RowInTile::Seven;
        if will_wrap {
            *self = RowInTile::Zero;
        } else {
            *self = FromPrimitive::from_u8(*self as u8 + 1).unwrap();
        }

        will_wrap
    }

    pub fn decrement(self) -> RowInTile {
        FromPrimitive::from_u8((self as u8).wrapping_sub(1)).unwrap()
    }
}
