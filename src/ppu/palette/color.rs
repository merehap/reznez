use enum_iterator::Sequence;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use splitbits::{combinebits, splitbits_ux};
use ux::{u2, u4, u6};

#[derive(PartialOrd, Ord, PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct Color {
    brightness: Brightness,
    hue: Hue,
}

impl Color {
    pub const BLACK: Self = Self { hue: Hue::Black, brightness: Brightness::Low };

    pub const fn new(brightness: Brightness, hue: Hue) -> Self {
        Self { brightness, hue }
    }

    pub fn to_greyscale(self) -> Self {
        Self { hue: Hue::Gray, brightness: self.brightness }
    }

    pub fn to_u6(self) -> u6 {
        u6::new(combinebits!(self.brightness as u8, self.hue as u8, "00bb hhhh"))
    }

    pub fn to_usize(self) -> usize {
        u8::from(self.to_u6()).into()
    }
}

impl From<u8> for Color {
    fn from(value: u8) -> Self {
        debug_assert_eq!(value & 0b1100_0000, 0, "First two bits must be 0.");

        let fields = splitbits_ux!(value, "..bb hhhh");
        Self {
            brightness: fields.b.into(),
            hue: fields.h.into(),
        }
    }
}

#[derive(PartialOrd, Ord, PartialEq, Eq, Hash, Clone, Copy, Debug, FromPrimitive, Sequence)]
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

impl From<u4> for Hue {
    fn from(value: u4) -> Self {
        FromPrimitive::from_u8(value.into()).unwrap()
    }
}

#[derive(PartialOrd, Ord, PartialEq, Eq, Hash, Clone, Copy, Debug, FromPrimitive, Sequence)]
pub enum Brightness {
    Minimum,
    Low,
    High,
    Maximum,
}

impl From<u2> for Brightness {
    fn from(value: u2) -> Self {
        FromPrimitive::from_u8(value.into()).unwrap()
    }
}