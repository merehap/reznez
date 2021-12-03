pub enum PaletteTableIndex {
    Zero,
    One,
    Two,
    Three,
}

impl PaletteTableIndex {
    pub fn unpack_byte(value: u8) -> [PaletteTableIndex; 4] {
        [
            PaletteTableIndex::from_low_bits(value >> 6),
            PaletteTableIndex::from_low_bits(value >> 4),
            PaletteTableIndex::from_low_bits(value >> 2),
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
