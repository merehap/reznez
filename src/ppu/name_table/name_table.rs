use std::fmt;

use enum_iterator::IntoEnumIterator;

use crate::ppu::pixel_index::{PixelRow, ColumnInTile};
use crate::ppu::name_table::background_tile_index::{BackgroundTileIndex, TileColumn, TileRow};
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
    tiles: &'a [u8; ATTRIBUTE_START_INDEX],
    attribute_table: AttributeTable<'a>,
}

impl <'a> NameTable<'a> {
    pub fn new(raw: &'a [u8; NAME_TABLE_SIZE]) -> NameTable<'a> {
        NameTable {
            tiles: raw[0..ATTRIBUTE_START_INDEX].try_into().unwrap(),
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
            self.render_scanline(pixel_row, pattern_table, palette_table, 0, 0, frame);
        }
    }

    pub fn render_scanline(
        &self,
        pixel_row: PixelRow,
        pattern_table: &PatternTable,
        palette_table: &PaletteTable,
        x_offset: i16,
        y_offset: i16,
        frame: &mut Frame,
    ) {
        let (tile_row, row_in_tile) = TileRow::from_pixel_row(pixel_row);
        let mut tile_sliver = [Rgbt::Transparent; 8];
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
                let maybe_pixel_column = background_tile_index
                    .tile_column()
                    .to_pixel_column(column_in_tile)
                    .offset(x_offset);
                if let Some(pixel_column) = maybe_pixel_column {
                    if let Some(pixel_row) = pixel_row.wrapping_offset(y_offset) {
                        frame.set_background_pixel(
                            pixel_column,
                            pixel_row,
                            tile_sliver[column_in_tile as usize],
                        );
                    }
                }
            }
        }
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
