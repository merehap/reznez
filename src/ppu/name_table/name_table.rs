use std::fmt;

use crate::memory::ppu::ppu_address::{XScroll, YScroll};
use crate::ppu::pixel_index::{PixelColumn, PixelRow};
use crate::ppu::name_table::background_tile_index::BackgroundTileIndex;
use crate::ppu::name_table::attribute_table::AttributeTable;
use crate::ppu::palette::palette_table::PaletteTable;
use crate::ppu::palette::palette_table_index::PaletteTableIndex;
use crate::ppu::palette::rgbt::Rgbt;
use crate::ppu::pattern_table::{PatternTable, PatternIndex};
use crate::ppu::render::frame::Frame;

const NAME_TABLE_SIZE: usize = 0x400;
const ATTRIBUTE_START_INDEX: usize = 0x3C0;

#[derive(Debug)]
pub struct NameTable<'a> {
    tiles: &'a [u8; NAME_TABLE_SIZE],
    attribute_table: AttributeTable<'a>,
}

impl <'a> NameTable<'a> {
    pub fn new(raw: &'a [u8; NAME_TABLE_SIZE]) -> NameTable<'a> {
        NameTable {
            tiles: raw,
            attribute_table:
                AttributeTable::new(raw[ATTRIBUTE_START_INDEX..].try_into().unwrap()),
        }
    }

    pub fn render(
        &self,
        pattern_table: &PatternTable,
        palette_table: &PaletteTable,
        frame: &mut Frame,
    ) {
        for pixel_row in PixelRow::iter() {
            self.render_scanline(
                pixel_row,
                pattern_table,
                palette_table,
                XScroll::ZERO,
                YScroll::ZERO,
                Rectangle::FULL,
                frame,
            );
        }
    }

    pub fn render_scanline(
        &self,
        pixel_row: PixelRow,
        pattern_table: &PatternTable,
        palette_table: &PaletteTable,
        x_scroll: XScroll,
        y_scroll: YScroll,
        bounds: Rectangle,
        frame: &mut Frame,
    ) {
        /*
        let original_pixel_row = pixel_row;
        let Some(pixel_row) = pixel_row.offset(y_offset) else {
            return;
        };
        */

        for pixel_column in PixelColumn::iter() {
            if bounds.is_in_bounds(pixel_column, pixel_row) {
                self.render_pixel(
                    pixel_column,
                    pixel_row,
                    pattern_table,
                    palette_table,
                    x_scroll,
                    y_scroll,
                    frame,
                );
            }
        }

        /*
        let (tile_row, row_in_tile) = TileRow::from_pixel_row(original_pixel_row);
        for tile_column in TileColumn::iter() {
            let background_tile_index =
                BackgroundTileIndex::from_tile_column_row(tile_column, tile_row);
            let (pattern_index, palette_table_index) =
                self.tile_entry_at(background_tile_index);
            pattern_table.render_background_tile_sliver(
                pattern_index,
                row_in_tile,
                palette_table.background_palette(palette_table_index),
                &mut tile_sliver,
            );

            for column_in_tile in ColumnInTile::into_enum_iter() {
                let pixel_column = background_tile_index
                    .tile_column()
                    .to_pixel_column(column_in_tile);
                if !bounds.is_in_bounds(pixel_column, pixel_row) {
                    continue;
                }

                let maybe_pixel_column = background_tile_index
                    .tile_column()
                    .to_pixel_column(column_in_tile)
                    .offset(x_offset);
                if let Some(pixel_column) = maybe_pixel_column {
                    frame.set_background_pixel(
                        pixel_column,
                        pixel_row,
                        tile_sliver[column_in_tile as usize],
                    );
                }
            }
        }
        */
    }

    fn render_pixel(
        &self,
        pixel_column: PixelColumn,
        pixel_row: PixelRow,
        pattern_table: &PatternTable,
        palette_table: &PaletteTable,
        x_scroll: XScroll,
        y_scroll: YScroll,
        frame: &mut Frame,
    ) {
        let (tile_column, column_in_tile) = x_scroll.tile_column(pixel_column);
        let (tile_row, row_in_tile) = y_scroll.tile_row(pixel_row);
        let background_tile_index =
            BackgroundTileIndex::from_tile_column_row(tile_column, tile_row);

        let (pattern_index, palette_table_index) =
            self.tile_entry_at(background_tile_index);
        let mut tile_sliver = [Rgbt::Transparent; 8];
        pattern_table.render_background_tile_sliver(
            pattern_index,
            row_in_tile,
            palette_table.background_palette(palette_table_index),
            &mut tile_sliver,
        );
        frame.set_background_pixel(
            pixel_column,
            pixel_row,
            tile_sliver[column_in_tile as usize],
        );
    }

    #[inline]
    fn tile_entry_at(
        &self,
        background_tile_index: BackgroundTileIndex,
    ) -> (PatternIndex, PaletteTableIndex) {

        let pattern_index =
            PatternIndex::new(self.tiles[background_tile_index.to_usize()]);
        let palette_table_index =
            self.attribute_table.palette_table_index(background_tile_index);

        (pattern_index, palette_table_index)
    }
}

impl fmt::Display for NameTable<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Nametable!")?;
        for index in BackgroundTileIndex::iter() {
            write!(f, "{:02X} ", self.tile_entry_at(index).0.to_usize())?;

            if index.tile_column().is_max() {
                writeln!(f)?;
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct Rectangle {
    left_column: PixelColumn,
    top_row: PixelRow,

    right_column: PixelColumn,
    bottom_row: PixelRow,
}

impl Rectangle {
    pub const FULL: Rectangle = Rectangle::const_from_raw((0, 0), (255, 239)).unwrap();

    pub const fn const_from_raw(
        (left, top): (u8, u8),
        (right, bottom): (u8, u8),
    ) -> Option<Rectangle> {
        if left > right || top > bottom {
            panic!();
        }

        let Some(top_row) = PixelRow::try_from_u8(top) else {
            return None;
        };

        let bottom_row = PixelRow::saturate_from_u8(bottom);
        let left_column = PixelColumn::new(left);
        let right_column = PixelColumn::new(right);
        Some(Rectangle {left_column, top_row, right_column, bottom_row})
    }

    pub fn from_raw(
        (left, top): (u8, u8),
        (right, bottom): (u8, u8),
    ) -> Option<Rectangle> {
        if left > right || top > bottom {
            panic!("Left: {}, Right: {}, Top: {}, Bottom: {}", left, right, top, bottom);
        }

        let Some(top_row) = PixelRow::try_from_u8(top) else {
            return None;
        };

        let bottom_row = PixelRow::saturate_from_u8(bottom);
        let left_column = PixelColumn::new(left);
        let right_column = PixelColumn::new(right);
        Some(Rectangle {left_column, top_row, right_column, bottom_row})
    }

    pub fn is_in_bounds(&self, column: PixelColumn, row: PixelRow) -> bool {
        self.left_column <= column && column <= self.right_column &&
            self.top_row <= row && row <= self.bottom_row
    }
}
