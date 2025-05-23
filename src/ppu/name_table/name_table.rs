use std::fmt;

use crate::memory::ppu::ppu_address::{XScroll, YScroll};
use crate::ppu::name_table::attribute_table::AttributeTable;
use crate::ppu::name_table::background_tile_index::BackgroundTileIndex;
use crate::ppu::palette::palette_table::PaletteTable;
use crate::ppu::palette::palette_table_index::PaletteTableIndex;
use crate::ppu::palette::rgbt::Rgbt;
use crate::ppu::pattern_table::{PatternIndex, PatternTable};
use crate::ppu::pixel_index::{ColumnInTile, PixelColumn, PixelRow, RowInTile};
use crate::ppu::render::frame::Frame;
use crate::util::unit::KIBIBYTE;

// The size of the name table proper plus attribute table.
const NAME_TABLE_SIZE: u32 = KIBIBYTE;
const ATTRIBUTE_TABLE_SIZE: u32 = 64;
// 0x3C0
pub const ATTRIBUTE_START_INDEX: u32 = NAME_TABLE_SIZE - ATTRIBUTE_TABLE_SIZE;

#[derive(Debug)]
pub struct NameTable<'a> {
    tiles: &'a [u8; NAME_TABLE_SIZE as usize],
    attribute_table: AttributeTable<'a>,
}

impl<'a> NameTable<'a> {
    pub fn new(raw: &'a [u8; NAME_TABLE_SIZE as usize]) -> NameTable<'a> {
        NameTable {
            tiles: raw,
            attribute_table: AttributeTable::new(
                raw[ATTRIBUTE_START_INDEX as usize..].try_into().unwrap(),
            ),
        }
    }

    pub fn attribute_table(&self) -> &AttributeTable<'a> {
        &self.attribute_table
    }

    pub fn tile_entry_for_pixel(
        &self,
        pixel_column: PixelColumn,
        pixel_row: PixelRow,
        x_scroll: XScroll,
        y_scroll: YScroll,
    ) -> (PatternIndex, PaletteTableIndex, ColumnInTile, RowInTile) {
        let (tile_column, column_in_tile) = x_scroll.tile_column(pixel_column);
        let (tile_row, row_in_tile) = y_scroll.tile_row(pixel_row);
        let background_tile_index =
            BackgroundTileIndex::from_tile_column_row(tile_column, tile_row);

        let (pattern_index, palette_table_index) =
            self.tile_entry_at(background_tile_index);
        (
            pattern_index,
            palette_table_index,
            column_in_tile,
            row_in_tile,
        )
    }

    #[inline]
    fn tile_entry_at(
        &self,
        background_tile_index: BackgroundTileIndex,
    ) -> (PatternIndex, PaletteTableIndex) {
        let pattern_index =
            PatternIndex::new(self.tiles[background_tile_index.to_usize()]);
        let palette_table_index = self
            .attribute_table
            .palette_table_index(background_tile_index.tile_column(), background_tile_index.tile_row());

        (pattern_index, palette_table_index)
    }
}

/**
 * DEBUG WINDOW METHODS
 */
impl NameTable<'_> {
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
                frame,
            );
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn render_scanline(
        &self,
        pixel_row: PixelRow,
        pattern_table: &PatternTable,
        palette_table: &PaletteTable,
        x_scroll: XScroll,
        y_scroll: YScroll,
        frame: &mut Frame,
    ) {
        for pixel_column in PixelColumn::iter() {
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

    #[allow(clippy::too_many_arguments)]
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
        pattern_table.render_pixel_sliver(
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

}

impl fmt::Display for NameTable<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Nametable!")?;
        for index in BackgroundTileIndex::iter() {
            write!(f, "{:02X} ", u16::from(self.tile_entry_at(index).0))?;

            if index.tile_column().is_max() {
                writeln!(f)?;
            }
        }

        Ok(())
    }
}
