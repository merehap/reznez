use std::fmt;

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
    tiles: &'a [u8; ATTRIBUTE_START_INDEX],
    attribute_table: AttributeTable<'a>,
}

impl <'a> NameTable<'a> {
    pub fn new(raw: &'a [u8; NAME_TABLE_SIZE]) -> NameTable<'a> {
        NameTable {
            tiles: (&raw[0..ATTRIBUTE_START_INDEX]).try_into().unwrap(),
            attribute_table:
                AttributeTable::new((&raw[ATTRIBUTE_START_INDEX..]).try_into().unwrap()),
        }
    }

    pub fn render(
        &self,
        pattern_table: &PatternTable,
        palette_table: &PaletteTable,
        x_offset: i16,
        y_offset: i16,
        frame: &mut Frame,
    ) {
        let mut tile_sliver = [Rgbt::Transparent; 8];
        for background_tile_index in BackgroundTileIndex::iter() {
            let (pattern_index, palette_table_index) =
                self.tile_entry_at(background_tile_index);
            for row_in_tile in 0..8 {
                pattern_table.render_tile_sliver(
                    pattern_index,
                    row_in_tile as usize,
                    palette_table.background_palette(palette_table_index),
                    &mut tile_sliver,
                );

                for column_in_tile in 0..8 {
                    let column = 8 * background_tile_index.column() as i16 + column_in_tile;
                    let column: Result<u8, _> = (column + x_offset).try_into();
                    let row = 8 * background_tile_index.row() as i16 + row_in_tile;
                    let row = ((row + y_offset).rem_euclid(256) as u8) % 240;
                    if let Ok(column) = column {
                        frame.background_row(row)[column as usize] =
                            tile_sliver[column_in_tile as usize];
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

            if index.to_usize() % 32 == 31 {
                writeln!(f)?;
            }
        }

        Ok(())
    }
}
