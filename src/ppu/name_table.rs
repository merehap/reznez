use crate::ppu::palette::palette_table_index::PaletteTableIndex;
use crate::ppu::tile_number::TileNumber;

pub struct NameTable<'a>(&'a [u8; 0x400]);

impl <'a> NameTable<'a> {
    pub fn new(raw: &'a [u8; 0x400]) -> NameTable<'a> {
        NameTable(raw)
    }

    pub fn tile_entry_at(&self, tile_number: TileNumber) -> (u8, [PaletteTableIndex; 4]) {
        let tile_entry = self.0[tile_number.to_usize()];
        let palette_entry = self.attribute_table()
            .palette_table_indexes_for_attribute(tile_number.to_usize() / 4);
        (tile_entry, palette_entry)
    }

    fn attribute_table(&self) -> AttributeTable<'a> {
        AttributeTable::new((&self.0[0x3C0..]).try_into().unwrap())
    }
}

struct AttributeTable<'a>(&'a [u8; 64]);

impl <'a> AttributeTable<'a> {
    pub fn new(raw: &'a [u8; 64]) -> AttributeTable<'a> {
        AttributeTable(raw)
    }

    pub fn palette_table_indexes_for_attribute(
        &self,
        attribute_index: usize,
    ) -> [PaletteTableIndex; 4] {

        PaletteTableIndex::unpack_byte(self.0[attribute_index])
    }
}
