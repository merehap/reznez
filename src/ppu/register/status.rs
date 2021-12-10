use crate::util::get_bit;

#[derive(Clone, Copy)]
pub struct Status {
    vblank_active: bool,
    sprite0_hit: bool,
    sprite_overflow: bool,
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

    pub fn vblank_active(self) -> bool {
        self.vblank_active
    }

    pub fn sprite0_hit(self) -> bool {
        self.sprite0_hit
    }

    pub fn sprite_overflow(self) -> bool {
        self.sprite_overflow
    }
}
