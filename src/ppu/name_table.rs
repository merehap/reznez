use std::fmt;

use crate::ppu::palette::palette_table_index::PaletteTableIndex;
use crate::ppu::pattern_table::PatternIndex;

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
            tiles: raw[..ATTRIBUTE_START_INDEX].try_into().unwrap(),
            attribute_table:
                AttributeTable(raw[ATTRIBUTE_START_INDEX..].try_into().unwrap()),
        }
    }

    #[inline]
    pub fn tile_entry_at(
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
            write!(f, "#{:02X} ", self.tile_entry_at(index).0.to_usize())?;
        }

        Ok(())
    }
}

#[derive(Debug)]
struct AttributeTable<'a>(&'a [u8; NAME_TABLE_SIZE - ATTRIBUTE_START_INDEX]);

impl <'a> AttributeTable<'a> {
    #[inline]
    fn palette_table_index(
        &self,
        background_tile_index: BackgroundTileIndex,
        ) -> PaletteTableIndex {

        let attribute_index =
            8 * (background_tile_index.row() / 4) +
            (background_tile_index.column() / 4);
        let attribute = self.0[attribute_index as usize];
        let palette_table_indexes = PaletteTableIndex::unpack_byte(attribute);
        let index_selection =
            if background_tile_index.row()    / 2 % 2 == 0 {2} else {0} +
            if background_tile_index.column() / 2 % 2 == 0 {1} else {0};
        palette_table_indexes[index_selection]
    }
}

const COLUMN_COUNT: u16 = 32;
const ROW_COUNT: u16 = 30;
const MAX_INDEX: u16 = COLUMN_COUNT * ROW_COUNT - 1;

#[derive(Clone, Copy)]
pub struct BackgroundTileIndex(u16);

impl BackgroundTileIndex {
    pub fn from_u16(number: u16) -> Result<BackgroundTileIndex, String> {
        if number > MAX_INDEX {
            return Err(format!(
                "Background tile index must not be greater than {}.",
                MAX_INDEX,
            ));
        }

        Ok(BackgroundTileIndex(number))
    }

    pub fn iter() -> BackgroundTileIndexIterator {
        BackgroundTileIndexIterator {index: BackgroundTileIndex(0)}
    }

    pub fn to_usize(self) -> usize {
        self.0 as usize
    }

    #[inline]
    pub fn column(self) -> u8 {
        (self.0 % 32).try_into().unwrap()
    }

    #[inline]
    pub fn row(self) -> u8 {
        (self.0 / 32).try_into().unwrap()
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
