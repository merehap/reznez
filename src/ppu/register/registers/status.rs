use crate::util::bit_util::pack_bools;

#[derive(Clone, Copy)]
pub struct Status {
    pub vblank_active: bool,
    pub sprite0_hit: bool,
    pub sprite_overflow: bool,
}

impl Status {
    pub fn new() -> Status {
        Status {
            vblank_active: false,
            sprite0_hit: false,
            sprite_overflow: false,
        }
    }

    pub fn to_u8(self) -> u8 {
        pack_bools(
            [
                self.vblank_active,
                self.sprite0_hit,
                self.sprite_overflow,
                false,
                false,
                false,
                false,
                false,
            ]
        )
    }
}
