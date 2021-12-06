use enum_iterator::IntoEnumIterator;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

#[derive(PartialOrd, Ord, PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct Color {
    hue: Hue,
    brightness: Brightness,
}

impl Color {
    pub const BLACK: Color = Color::new(Hue::Black, Brightness::Minimum);

    pub const fn new(hue: Hue, brightness: Brightness) -> Color {
        Color {hue, brightness}
    }

    pub fn from_u8(value: u8) -> Result<Color, String> {
        if value & 0b1100_0000 == 0 {
            Ok(Color {
                hue: FromPrimitive::from_u8(value & 0b0000_1111).unwrap(),
                brightness: FromPrimitive::from_u8((value & 0b0011_0000) >> 4).unwrap(),
            })
        } else {
            Err("The top two bits for a color should not be set.".to_string())
        }
    }
}

#[derive(PartialOrd, Ord, PartialEq, Eq, Hash, Clone, Copy, Debug,
         FromPrimitive, IntoEnumIterator)]
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

#[derive(PartialOrd, Ord, PartialEq, Eq, Hash, Clone, Copy, Debug,
         FromPrimitive, IntoEnumIterator)]
pub enum Brightness {
    Minimum,
    Low,
    High,
    Maximum,
}
