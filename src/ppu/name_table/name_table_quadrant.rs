use  modular_bitfield::BitfieldSpecifier;
use num_derive::FromPrimitive;

use crate::util::bit_util::get_bit;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug, FromPrimitive, BitfieldSpecifier)]
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

    pub fn increment(&mut self) -> bool {
        use NameTableQuadrant::*;
        let (result, wrap) = match self {
            TopLeft     => (TopRight, false),
            TopRight    => (BottomLeft, false),
            BottomLeft  => (BottomRight, false),
            BottomRight => (TopLeft, true),
        };

        *self = result;
        wrap
    }

    pub fn copy_horizontal_side_from(&mut self, other: NameTableQuadrant) {
        let different_sides = self.is_on_left() != other.is_on_left();
        if different_sides {
            *self = self.next_horizontal();
        }
    }

    fn is_on_left(self) -> bool {
        use NameTableQuadrant::*;
        self == TopLeft || self == BottomLeft
    }
}
