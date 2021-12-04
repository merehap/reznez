use crate::ppu::palette::palette_table_index::PaletteTableIndex;
use crate::ppu::tile_number::TileNumber;

const NAME_TABLE_SIZE: usize = 0x400;
const ATTRIBUTE_START_INDEX: usize = 0x3C0;

#[derive(Debug)]
pub struct NameTable<'a> {
    tiles: &'a [u8; ATTRIBUTE_START_INDEX],
    attributes: &'a [u8; NAME_TABLE_SIZE - ATTRIBUTE_START_INDEX],
}

impl <'a> NameTable<'a> {
    pub fn new(raw: &'a [u8; NAME_TABLE_SIZE]) -> NameTable<'a> {
        NameTable {
            tiles: raw[..ATTRIBUTE_START_INDEX].try_into().unwrap(),
            attributes: raw[ATTRIBUTE_START_INDEX..].try_into().unwrap(),
        }
    }

    pub fn tile_entry_at(&self, tile_number: TileNumber) -> (u8, [PaletteTableIndex; 4]) {
        let tile_entry = self.tiles[tile_number.to_usize()];

        let attribute = self.attributes[tile_number.attribute_index()];
        let palette_table_indexes = PaletteTableIndex::unpack_byte(attribute);

        (tile_entry, palette_table_indexes)
    }
}
