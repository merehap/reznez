use crate::ppu::palette::rgb::Rgb;

#[derive(Clone, Copy)]
pub enum Rgbt {
    Transparent,
    Opaque(Rgb),
}

impl Rgbt {
    #[inline]
    pub fn opaque(red: u8, green: u8, blue: u8) -> Rgbt {
        Rgbt::Opaque(Rgb::new(red, green, blue))
    }
}
