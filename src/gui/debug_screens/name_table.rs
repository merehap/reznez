use std::fmt;

use crate::mapper::{Mapper, NameTableQuadrant};
use crate::memory::memory::Memory;
use crate::memory::ppu::ppu_address::{XScroll, YScroll};
use crate::gui::debug_screens::attribute_table::AttributeTable;
use crate::ppu::constants::{ATTRIBUTE_START_INDEX, NAME_TABLE_SIZE};
use crate::ppu::name_table::background_tile_index::BackgroundTileIndex;
use crate::ppu::palette::palette_table::PaletteTable;
use crate::ppu::palette::palette_table_index::PaletteTableIndex;
use crate::ppu::palette::rgbt::Rgbt;
use crate::gui::debug_screens::pattern_table::PatternTable;
use crate::ppu::pixel_index::{PixelColumn, PixelRow};
use crate::ppu::render::frame::Frame;
use crate::ppu::tile_number::TileNumber;

// Used for debug window purposes only. The actual rendering pipeline deals with unabstracted bytes.
#[derive(Debug)]
pub struct NameTable<'a> {
    tile_numbers: &'a [u8; NAME_TABLE_SIZE as usize],
    attribute_table: AttributeTable<'a>,
}

impl<'a> NameTable<'a> {
    pub fn new(raw: &'a [u8; NAME_TABLE_SIZE as usize]) -> NameTable<'a> {
        NameTable {
            tile_numbers: raw[0..ATTRIBUTE_START_INDEX as usize].try_into().unwrap(),
            attribute_table: AttributeTable::new(raw[ATTRIBUTE_START_INDEX as usize..].try_into().unwrap()),
        }
    }

    pub fn from_mem(mapper: &'a dyn Mapper, mem: &'a Memory, quadrant: NameTableQuadrant) -> NameTable<'a> {
        let mapper_params = mem.mapper_params();
        let ciram = mem.ciram();
        NameTable::new(mapper.raw_name_table(mapper_params, ciram, quadrant))
    }

    pub fn render(&self, pattern_table: &PatternTable, palette_table: &PaletteTable, frame: &mut Frame) {
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
        let background_tile_index = BackgroundTileIndex::from_tile_column_row(tile_column, tile_row);

        let (tile_number, palette_table_index) = self.tile_entry_at(background_tile_index);
        let mut tile_sliver = [Rgbt::Transparent; 8];
        pattern_table.render_pixel_sliver(
            tile_number,
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
    ) -> (TileNumber, PaletteTableIndex) {
        let tile_number = TileNumber::new(self.tile_numbers[background_tile_index.to_usize()]);
        let palette_table_index = self
            .attribute_table
            .palette_table_index(background_tile_index.tile_column(), background_tile_index.tile_row());

        (tile_number, palette_table_index)
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
