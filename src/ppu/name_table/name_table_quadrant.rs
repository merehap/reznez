// modular_bitfield pedantic clippy warnings
#![allow(clippy::cast_lossless, clippy::no_effect_underscore_binding, clippy::map_unwrap_or)]

use modular_bitfield::Specifier;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use ux::u2;

use crate::mapper::ChrBankRegisterId;
use crate::memory::bank::bank::ChrSourceRegisterId;
use crate::util::bit_util::get_bit;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug, FromPrimitive, Specifier)]
pub enum NameTableQuadrant {
    TopLeft = 0,
    TopRight = 1,
    BottomLeft = 2,
    BottomRight = 3,
}

#[rustfmt::skip]
impl NameTableQuadrant {
    pub const ALL: [Self; 4] = [Self::TopLeft, Self::TopRight, Self::BottomLeft, Self::BottomRight];

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

    pub fn register_ids(self) -> (ChrSourceRegisterId, ChrBankRegisterId) {
        use NameTableQuadrant::*;
        match self {
            TopLeft     => (ChrSourceRegisterId::NT0, ChrBankRegisterId::N0),
            TopRight    => (ChrSourceRegisterId::NT1, ChrBankRegisterId::N1),
            BottomLeft  => (ChrSourceRegisterId::NT2, ChrBankRegisterId::N2),
            BottomRight => (ChrSourceRegisterId::NT3, ChrBankRegisterId::N3),
        }
    }

    fn is_on_left(self) -> bool {
        use NameTableQuadrant::*;
        self == TopLeft || self == BottomLeft
    }
}

impl From<u2> for NameTableQuadrant {
    fn from(value: u2) -> Self {
        FromPrimitive::from_u8(value.into()).unwrap()
    }
}

impl From<NameTableQuadrant> for u16 {
    fn from(value: NameTableQuadrant) -> Self {
        value as u16
    }
}
