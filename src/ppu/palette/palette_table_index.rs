use enum_iterator::IntoEnumIterator;

use crate::ppu::name_table::background_tile_index::{TileColumn, TileRow};

#[derive(Clone, Copy, Debug, Default, IntoEnumIterator)]
pub enum PaletteTableIndex {
    #[default]
    Zero,
    One,
    Two,
    Three,
}

impl PaletteTableIndex {
    pub fn from_attribute_byte(
        attribute_byte: u8,
        tile_column: TileColumn,
        tile_row: TileRow,
    ) -> PaletteTableIndex {
        let palette_table_indexes = PaletteTableIndex::unpack_byte(attribute_byte);
        let index_selection =
            if tile_row.to_usize()    / 2 % 2 == 0 {2} else {0} +
            if tile_column.to_usize() / 2 % 2 == 0 {1} else {0};
        palette_table_indexes[index_selection]
    }

    pub fn unpack_byte(value: u8) -> [PaletteTableIndex; 4] {
        [
            // Bottom right.
            PaletteTableIndex::from_low_bits(value >> 6),
            // Bottom left.
            PaletteTableIndex::from_low_bits(value >> 4),
            // Top right.
            PaletteTableIndex::from_low_bits(value >> 2),
            // Top left.
            PaletteTableIndex::from_low_bits(value),
        ]
    }

    fn from_low_bits(value: u8) -> PaletteTableIndex {
        match value & 0b0000_0011 {
            0 => PaletteTableIndex::Zero,
            1 => PaletteTableIndex::One,
            2 => PaletteTableIndex::Two,
            3 => PaletteTableIndex::Three,
            _ => unreachable!(),
        }
    }
}
