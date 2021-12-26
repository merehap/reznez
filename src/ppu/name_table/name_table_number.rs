use num_derive::FromPrimitive;

#[derive(PartialEq, Eq, Clone, Copy, Debug, FromPrimitive)]
pub enum NameTableNumber {
    Zero,
    One,
    Two,
    Three,
}
