pub struct Color {
    hue: Hue,
    brightness: Brightness,
}

impl Color {
    pub fn new(value: u8) -> Color {
        assert!(value & 0b1100_0000 == 0);

        use Hue::*;
        let hue = match value & 0b0000_1111 {
            0x0 => Gray,
            0x1 => Azure,
            0x2 => Blue,
            0x3 => Violet,
            0x4 => Magenta,
            0x5 => Rose,
            0x6 => Maroon,
            0x7 => Orange,
            0x8 => Olive,
            0x9 => Chartreuse,
            0xA => Green,
            0xB => Spring,
            0xC => Cyan,
            0xD => DarkGray,
            0xE => Black,
            0xF => ExtraBlack,
            _ => unreachable!(),
        };

        let brightness = match (value & 0b0011_0000) >> 4 {
            0 => Brightness::Minimum,
            1 => Brightness::Low,
            2 => Brightness::High,
            3 => Brightness::Maximum,
            _ => unreachable!(),
        };

        Color {hue, brightness}
    }
}

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

pub enum Brightness {
    Minimum,
    Low,
    High,
    Maximum,
}
