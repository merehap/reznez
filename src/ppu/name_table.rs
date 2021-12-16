use std::fmt;

use crate::ppu::palette::palette_table_index::PaletteTableIndex;
use crate::ppu::tile_number::TileNumber;

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
    pub fn tile_entry_at(&self, tile_number: TileNumber) -> (u8, PaletteTableIndex) {
        let tile_entry = self.tiles[tile_number.to_usize()];
        let palette_table_index =
            self.attribute_table.palette_table_index(tile_number);

        (tile_entry, palette_table_index)
    }
}

impl fmt::Display for NameTable<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Nametable!")?;
        for tile_number in TileNumber::iter() {
            write!(f, "#{:02X} ", self.tile_entry_at(tile_number).0)?;
        }

        Ok(())
    }
}

#[derive(Debug)]
struct AttributeTable<'a>(&'a [u8; NAME_TABLE_SIZE - ATTRIBUTE_START_INDEX]);

impl <'a> AttributeTable<'a> {
    #[inline]
    fn palette_table_index(&self, tile_number: TileNumber) -> PaletteTableIndex {
        let attribute_index = 8 * (tile_number.row() / 4) + (tile_number.column() / 4);
        let attribute = self.0[attribute_index as usize];
        let palette_table_indexes = PaletteTableIndex::unpack_byte(attribute);
        let index_selection =
            if tile_number.row()    % 2 == 0 {2} else {0} +
            if tile_number.column() % 2 == 0 {1} else {0};
        palette_table_indexes[index_selection]
    }
}
