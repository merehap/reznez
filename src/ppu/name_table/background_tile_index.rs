use crate::ppu::pixel_index::{PixelColumn, ColumnInTile, PixelRow, RowInTile};

const COLUMN_COUNT: u16 = 32;
const ROW_COUNT: u16 = 30;
const MAX_INDEX: u16 = COLUMN_COUNT * ROW_COUNT - 1;

#[derive(Clone, Copy)]
pub struct BackgroundTileIndex(u16);

impl BackgroundTileIndex {
    pub fn iter() -> BackgroundTileIndexIterator {
        BackgroundTileIndexIterator {index: BackgroundTileIndex(0)}
    }

    pub fn to_usize(self) -> usize {
        self.0 as usize
    }

    #[inline]
    pub fn tile_column(self) -> TileColumn {
        TileColumn::try_from_u8((self.0 % 32) as u8).unwrap()
    }

    #[inline]
    pub fn tile_row(self) -> TileRow {
        TileRow::try_from_u8((self.0 / 32) as u8).unwrap()
    }
}

pub struct BackgroundTileIndexIterator {
    index: BackgroundTileIndex,
}

impl Iterator for BackgroundTileIndexIterator {
    type Item = BackgroundTileIndex;

    fn next(&mut self) -> Option<BackgroundTileIndex> {
        if self.index.0 > MAX_INDEX {
            return None;
        }

        let result = self.index;
        self.index.0 += 1;
        Some(result)
    }
}

#[derive(Clone, Copy)]
pub struct TileColumn(u8);

impl TileColumn {
    const MAX: u8 = 31;

    fn try_from_u8(tile_column: u8) -> Option<TileColumn> {
        if tile_column <= TileColumn::MAX {
            Some(TileColumn(tile_column))
        } else {
            None
        }
    }

    pub fn to_pixel_column(self, column_in_tile: ColumnInTile) -> PixelColumn {
        // Unwrap always succeeds since 248 + 7 == 255.
        self.pixel_column().add_column_in_tile(column_in_tile).unwrap()
    }

    pub fn to_u8(self) -> u8 {
        self.0
    }

    fn pixel_column(self) -> PixelColumn {
        PixelColumn::new(8 * self.0)
    }
}

#[derive(Clone, Copy)]
pub struct TileRow(u8);

impl TileRow {
    const MAX: u8 = 29;

    fn try_from_u8(tile_row: u8) -> Option<TileRow> {
        if tile_row <= TileRow::MAX {
            Some(TileRow(tile_row))
        } else {
            None
        }
    }

    pub fn to_pixel_row(self, row_in_tile: RowInTile) -> PixelRow {
        // Unwrap always succeeds since 232 + 7 == 239.
        self.pixel_row().add_row_in_tile(row_in_tile).unwrap()
    }

    pub fn to_u8(self) -> u8 {
        self.0
    }

    fn pixel_row(self) -> PixelRow {
        PixelRow::try_from_u8(8 * self.0).unwrap()
    }
}
