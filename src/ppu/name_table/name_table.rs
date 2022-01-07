use std::fmt;

use crate::ppu::name_table::background_tile_index::BackgroundTileIndex;
use crate::ppu::name_table::attribute_table::AttributeTable;
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
                AttributeTable::new(raw[ATTRIBUTE_START_INDEX..].try_into().unwrap()),
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
            write!(f, "{:02X} ", self.tile_entry_at(index).0.to_usize())?;

            if index.to_usize() % 32 == 31 {
                writeln!(f)?;
            }
        }

        Ok(())
    }
}
