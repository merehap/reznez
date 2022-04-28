use num_derive::FromPrimitive;

use crate::util::bit_util::get_bit;

#[derive(PartialEq, Eq, Clone, Copy, Debug, FromPrimitive)]
pub enum NameTablePosition {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

#[rustfmt::skip]
impl NameTablePosition {
    pub fn from_last_two_bits(value: u8) -> NameTablePosition {
        match (get_bit(value, 6), get_bit(value, 7)) {
            (false, false) => NameTablePosition::TopLeft,
            (false, true ) => NameTablePosition::TopRight,
            (true , false) => NameTablePosition::BottomLeft,
            (true , true ) => NameTablePosition::BottomRight,
        }
    }

    pub fn next_horizontal(self) -> NameTablePosition {
        use NameTablePosition::*;
        match self {
            TopLeft     => TopRight,
            TopRight    => TopLeft,
            BottomLeft  => BottomRight,
            BottomRight => BottomLeft,
        }
    }

    pub fn next_vertical(self) -> NameTablePosition {
        use NameTablePosition::*;
        match self {
            TopLeft     => BottomLeft,
            TopRight    => BottomRight,
            BottomLeft  => TopLeft,
            BottomRight => TopRight,
        }
    }
}
