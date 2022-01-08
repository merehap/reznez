use num_derive::FromPrimitive;

#[derive(PartialEq, Eq, Clone, Copy, Debug, FromPrimitive)]
pub enum NameTableNumber {
    Zero,
    One,
    Two,
    Three,
}

impl NameTableNumber {
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
