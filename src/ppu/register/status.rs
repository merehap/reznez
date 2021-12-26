use crate::util;
use crate::util::get_bit;

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

    pub fn from_u8(value: u8) -> Status {
        assert!(
            value & 0b0001_1111 != 0,
            "Expected none of the lower 5 bits to be set in PPU STATUS, but found 0x{:X}.",
            value
            );
        Status {
            vblank_active: get_bit(value, 0),
            sprite0_hit: get_bit(value, 1),
            sprite_overflow: get_bit(value, 2),
        }
    }

    pub fn to_u8(&self) -> u8 {
        util::pack_bools(
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
