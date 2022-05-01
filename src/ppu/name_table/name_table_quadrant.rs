use num_derive::FromPrimitive;

use crate::util::bit_util::get_bit;

#[derive(PartialEq, Eq, Clone, Copy, Debug, FromPrimitive)]
pub enum NameTableQuadrant {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

#[rustfmt::skip]
impl NameTableQuadrant {
    pub fn from_last_two_bits(value: u8) -> NameTableQuadrant {
        match (get_bit(value, 6), get_bit(value, 7)) {
            (false, false) => NameTableQuadrant::TopLeft,
            (false, true ) => NameTableQuadrant::TopRight,
            (true , false) => NameTableQuadrant::BottomLeft,
            (true , true ) => NameTableQuadrant::BottomRight,
        }
    }

    pub fn next_horizontal(self) -> NameTableQuadrant {
        use NameTableQuadrant::*;
        match self {
            TopLeft     => TopRight,
            TopRight    => TopLeft,
            BottomLeft  => BottomRight,
            BottomRight => BottomLeft,
        }
    }

    pub fn next_vertical(self) -> NameTableQuadrant {
        use NameTableQuadrant::*;
        match self {
            TopLeft     => BottomLeft,
            TopRight    => BottomRight,
            BottomLeft  => TopLeft,
            BottomRight => TopRight,
        }
    }
}
