use num_derive::FromPrimitive;

#[derive(Clone, Copy, FromPrimitive)]
pub enum NameTableNumber {
    Zero,
    One,
    Two,
    Three,
}
