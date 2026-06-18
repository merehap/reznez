use enum_iterator::Sequence;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use splitbits::{combinebits, splitbits};

#[derive(PartialOrd, Ord, PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct Color {
    brightness: Brightness,
    hue: Hue,
}

impl Color {
    pub const BLACK: Self = Self { hue: Hue::Black, brightness: Brightness::Low };

    pub const fn new(hue: Hue, brightness: Brightness) -> Self {
        Self { hue, brightness }
    }

    pub fn to_usize(self) -> usize {
        combinebits!(self.brightness as u8, self.hue as u8, "00bb hhhh") as usize
    }
}

impl From<u8> for Color {
    fn from(value: u8) -> Self {
        debug_assert_eq!(value & 0b1100_0000, 0, "First two bits must be 0.");

        let fields = splitbits!(value, "..bb hhhh");
        Self {
            hue: FromPrimitive::from_u8(fields.h).unwrap(),
            brightness: FromPrimitive::from_u8(fields.b).unwrap(),
        }
    }
}

#[derive(
    PartialOrd,
    Ord,
    PartialEq,
    Eq,
    Hash,
    Clone,
    Copy,
    Debug,
    FromPrimitive,
    Sequence,
)]
pub enum Hue {
    Gray,
    Azure,
    Blue,
    Violet,
    Magenta,
    Rose,
    Maroon,
    Orange,
    Olive,
    Chartreuse,
    Green,
    Spring,
    Cyan,
    DarkGray,
    Black,
    ExtraBlack,
}

#[derive(
    PartialOrd,
    Ord,
    PartialEq,
    Eq,
    Hash,
    Clone,
    Copy,
    Debug,
    FromPrimitive,
)]
pub enum Brightness {
    Minimum,
    Low,
    High,
    Maximum,
}
