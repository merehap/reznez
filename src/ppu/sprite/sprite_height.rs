use crate::ppu::sprite::sprite_half::SpriteHalf;

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum SpriteHeight {
    Normal = 8,
    Tall = 16,
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
}