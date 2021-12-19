use std::fmt;

use crate::ppu::palette::palette_table_index::PaletteTableIndex;
use crate::ppu::name_table::background_tile_index::BackgroundTileIndex;

const ATTRIBUTE_TABLE_SIZE: usize = 64;

#[derive(Debug)]
pub struct AttributeTable<'a>(&'a [u8; ATTRIBUTE_TABLE_SIZE]);

impl <'a> AttributeTable<'a> {
    pub fn new(raw: &'a [u8; ATTRIBUTE_TABLE_SIZE]) -> AttributeTable<'a> {
        AttributeTable(raw)
    }

    #[inline]
    pub fn palette_table_index(
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

impl fmt::Display for AttributeTable<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Attribute Table!")?;
        for index in BackgroundTileIndex::iter() {
            write!(f, "{} ", self.palette_table_index(index) as usize)?;

            if index.to_usize() % 32 == 31 {
                writeln!(f)?;
            }
        }

        Ok(())
    }
}
