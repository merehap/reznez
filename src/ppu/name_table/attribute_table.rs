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

        // TODO: This should all be calculations within BackgroundTileIndex.
        let tile_column = background_tile_index.tile_column().to_u8();
        let tile_row = background_tile_index.tile_row().to_u8();

        let attribute_index =
            8 * (tile_row / 4) +
            (tile_column / 4);
        let attribute = self.0[attribute_index as usize];
        let palette_table_indexes = PaletteTableIndex::unpack_byte(attribute);
        let index_selection =
            if tile_row    / 2 % 2 == 0 {2} else {0} +
            if tile_column / 2 % 2 == 0 {1} else {0};
        palette_table_indexes[index_selection]
    }
}

impl fmt::Display for AttributeTable<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Attribute Table!")?;
        for index in BackgroundTileIndex::iter() {
            write!(f, "{} ", self.palette_table_index(index) as usize)?;

            if index.tile_column().is_max() {
                writeln!(f)?;
            }
        }

        Ok(())
    }
}
