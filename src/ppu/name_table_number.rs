use num_derive::FromPrimitive;

#[derive(Clone, Copy, Debug, FromPrimitive)]
pub enum NameTableNumber {
    Zero,
    One,
    Two,
    Three,
}
