#![allow(dead_code)]
// modular_bitfield pedantic clippy warnings
#![allow(clippy::cast_lossless, clippy::no_effect_underscore_binding)]

use modular_bitfield::bitfield;
use modular_bitfield::specifiers::B5;

#[bitfield]
#[derive(Clone, Copy)]
pub struct Status {
    #[skip]
    _unused: B5,

    pub sprite_overflow: bool,
    pub sprite0_hit: bool,
    pub vblank_active: bool,
}

impl Status {
    pub fn to_u8(self) -> u8 {
        self.into_bytes()[0]
    }
}
