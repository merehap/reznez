// modular_bitfield pedantic clippy warnings
#![allow(clippy::cast_lossless, clippy::no_effect_underscore_binding)]

use modular_bitfield::Specifier;

use crate::ppu::sprite::sprite_half::SpriteHalf;

#[derive(PartialEq, Eq, Clone, Copy, Debug, Specifier)]
pub enum SpriteHeight {
    Normal,
    Tall,
}

impl SpriteHeight {
    #[rustfmt::skip]
    pub fn sprite_half(self, y_offset: u8) -> Option<SpriteHalf> {
        match (self, y_offset / 8) {
            (_                 , 0) => Some(SpriteHalf::Top),
            (SpriteHeight::Tall, 1) => Some(SpriteHalf::Bottom),
            (_                 , _) => None,
        }
    }

    pub fn is_in_range(self, y_offset: u8) -> bool {
        self.sprite_half(y_offset).is_some()
    }

    pub fn to_dimension(self) -> u8 {
        match self {
            SpriteHeight::Normal => 8,
            SpriteHeight::Tall => 16,
        }
    }
}
