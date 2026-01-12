use splitbits::combinebits;

#[derive(Clone, Copy, Default)]
pub struct Status {
    pub vblank_active: bool,
    pub sprite0_hit: bool,
    pub sprite_overflow: bool,
}

impl Status {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn to_u8(self) -> u8 {
        combinebits!(self.vblank_active, self.sprite0_hit, self.sprite_overflow, "vso00000")
    }
}
