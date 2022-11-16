use num_derive::FromPrimitive;

#[derive(Clone, Copy, FromPrimitive)]
pub enum SpriteHalf {
    Top,
    Bottom,
}

impl SpriteHalf {
    pub fn flip(self) -> SpriteHalf {
        use SpriteHalf::*;
        match self {
            Top    => Bottom,
            Bottom => Top,
        }
    }
}
