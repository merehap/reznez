#[derive(Clone, Copy, Debug)]
pub enum PaletteIndex {
    One = 0,
    Two = 1,
    Three = 2,
}

impl PaletteIndex {
    pub fn from_two_low_bits(value: u8) -> Option<PaletteIndex> {
        match value & 0b11 {
            0b00 => None,
            0b01 => Some(PaletteIndex::One),
            0b10 => Some(PaletteIndex::Two),
            0b11 => Some(PaletteIndex::Three),
            _ => unreachable!(),
        }
    }

    pub fn from_low_high(low_bit: bool, high_bit: bool) -> Option<PaletteIndex> {
        match (low_bit, high_bit) {
            (false, false) => None,
            (true, false) => Some(PaletteIndex::One),
            (false, true) => Some(PaletteIndex::Two),
            (true, true) => Some(PaletteIndex::Three),
        }
    }
}
