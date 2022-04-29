use crate::ppu::palette::rgb::Rgb;

#[derive(Clone, Copy, Debug)]
pub enum Rgbt {
    Transparent,
    Opaque(Rgb),
}

impl Rgbt {
    #[inline]
    pub fn opaque(red: u8, green: u8, blue: u8) -> Rgbt {
        Rgbt::Opaque(Rgb::new(red, green, blue))
    }

    pub fn is_transparent(self) -> bool {
        matches!(self, Rgbt::Transparent)
    }

    pub fn is_opaque(self) -> bool {
        matches!(self, Rgbt::Opaque(_))
    }

    pub fn to_rgb(self) -> Option<Rgb> {
        if let Rgbt::Opaque(rgb) = self {
            Some(rgb)
        } else {
            None
        }
    }
}
