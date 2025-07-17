// modular_bitfield pedantic clippy warnings
#![allow(clippy::cast_lossless, clippy::no_effect_underscore_binding, clippy::map_unwrap_or, clippy::semicolon_if_nothing_returned)]
#![allow(unused_parens)]

use modular_bitfield::{bitfield, Specifier};

use crate::ppu::name_table::name_table_quadrant::NameTableQuadrant;
use crate::ppu::pattern_table_side::PatternTableSide;
use crate::ppu::sprite::sprite_height::SpriteHeight;

#[bitfield]
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct Ctrl {
    pub base_name_table_quadrant: NameTableQuadrant,
    pub current_address_increment: AddressIncrement,
    pub sprite_table_side: PatternTableSide,
    pub background_table_side: PatternTableSide,
    pub sprite_height: SpriteHeight,
    #[allow(dead_code)]
    pub ext_pin_role: ExtPinRole,
    pub nmi_enabled: bool,
}

impl Ctrl {
    pub fn from_u8(value: u8) -> Ctrl {
        Ctrl::from_bytes([value])
    }

    #[allow(dead_code)]
    pub fn to_u8(self) -> u8 {
        self.into_bytes()[0]
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug, Specifier)]
pub enum ExtPinRole {
    Read,
    Write,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug, Specifier)]
pub enum AddressIncrement {
    Right,
    Down,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        for i in 0..=255 {
            // Wide sprites not supported yet so zero-out that bit.
            let i = i & 0b1101_1111;
            assert_eq!(i, Ctrl::from_u8(i).to_u8());
        }
    }

    #[test]
    fn roundtrip_new() {
        let ctrl = Ctrl::new();
        assert_eq!(ctrl.to_u8(), 0);
        assert_eq!(ctrl, Ctrl::from_u8(ctrl.to_u8()));
    }
}
