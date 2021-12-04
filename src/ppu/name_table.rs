use crate::ppu::palette::palette_table_index::PaletteTableIndex;
use crate::ppu::tile_number::TileNumber;

#[derive(Debug)]
pub struct NameTable<'a>(&'a [u8; 0x400]);

impl <'a> NameTable<'a> {
    pub fn new(raw: &'a [u8; 0x400]) -> NameTable<'a> {
        NameTable(raw)
    }

    pub fn tile_entry_at(&self, tile_number: TileNumber) -> (u8, [PaletteTableIndex; 4]) {
        let tile_entry = self.0[tile_number.to_usize()];
        // The end of the name table is the attribute table,
        // which determines which palettes to use.
        let attribute_index = tile_number.to_usize() / 4 + 0x3C0;
        let palette_table_indexes =
            PaletteTableIndex::unpack_byte(self.0[attribute_index]);
        (tile_entry, palette_table_indexes)
    }
}
