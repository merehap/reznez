use crate::ppu::palette::color::Color;

#[derive(Clone, Copy, Debug)]
pub enum ColorT {
    Transparent,
    Opaque(Color),
}

impl ColorT {
    #[inline]
    pub fn opaque(color: Color) -> Self {
        ColorT::Opaque(color)
    }

    pub fn is_transparent(self) -> bool {
        matches!(self, ColorT::Transparent)
    }

    pub fn is_opaque(self) -> bool {
        matches!(self, ColorT::Opaque(_))
    }

    pub fn to_color(self) -> Option<Color> {
        if let ColorT::Opaque(rgb) = self {
            Some(rgb)
        } else {
            None
        }
    }
}