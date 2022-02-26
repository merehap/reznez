use num_derive::FromPrimitive;

use crate::util::bit_util::get_bit;

#[derive(PartialEq, Eq, Clone, Copy, Debug, FromPrimitive)]
pub enum NameTableNumber {
    Zero,
    One,
    Two,
    Three,
}

impl NameTableNumber {
    pub fn from_last_two_bits(value: u8) -> NameTableNumber {
        match (get_bit(value, 6), get_bit(value, 7)) {
            (false, false) => NameTableNumber::Zero,
            (false, true ) => NameTableNumber::One,
            (true , false) => NameTableNumber::Two,
            (true , true ) => NameTableNumber::Three,
        }
    }

    pub fn next_horizontal(self) -> NameTableNumber {
        use NameTableNumber::*;
        match self {
            Zero  => One,
            One   => Zero,
            Two   => Three,
            Three => Two,
        }
    }

    pub fn next_vertical(self) -> NameTableNumber {
        use NameTableNumber::*;
        match self {
            Zero  => Two,
            One   => Three,
            Two   => Zero,
            Three => One,
        }
    }
}
