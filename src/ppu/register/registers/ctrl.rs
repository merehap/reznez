use crate::ppu::name_table::name_table_position::NameTablePosition;
use crate::ppu::pattern_table::PatternTableSide;
use crate::util::bit_util::{get_bit, pack_bools};

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct Ctrl {
    pub nmi_enabled: bool,
    pub ext_pin_role: ExtPinRole,
    pub sprite_height: SpriteHeight,
    pub background_table_side: PatternTableSide,
    pub sprite_table_side: PatternTableSide,
    pub current_address_increment: AddressIncrement,
    pub base_name_table_position: NameTablePosition,
}

impl Ctrl {
    pub fn new() -> Ctrl {
        Ctrl {
            nmi_enabled: false,
            ext_pin_role: ExtPinRole::Read,
            sprite_height: SpriteHeight::Normal,
            background_table_side: PatternTableSide::Left,
            sprite_table_side: PatternTableSide::Left,
            current_address_increment: AddressIncrement::Right,
            base_name_table_position: NameTablePosition::TopLeft,
        }
    }

    pub fn from_u8(value: u8) -> Ctrl {
        Ctrl {
            nmi_enabled: get_bit(value, 0),
            ext_pin_role: if get_bit(value, 1) {
                ExtPinRole::Write
            } else {
                ExtPinRole::Read
            },
            sprite_height: if get_bit(value, 2) {
                SpriteHeight::Tall
            } else {
                SpriteHeight::Normal
            },
            background_table_side: if get_bit(value, 3) {
                PatternTableSide::Right
            } else {
                PatternTableSide::Left
            },
            sprite_table_side: if get_bit(value, 4) {
                PatternTableSide::Right
            } else {
                PatternTableSide::Left
            },
            current_address_increment: if get_bit(value, 5) {
                AddressIncrement::Down
            } else {
                AddressIncrement::Right
            },
            base_name_table_position: NameTablePosition::from_last_two_bits(value),
        }
    }

    #[allow(dead_code)]
    pub fn to_u8(self) -> u8 {
        pack_bools([
            self.nmi_enabled,
            self.ext_pin_role == ExtPinRole::Write,
            self.sprite_height == SpriteHeight::Tall,
            self.background_table_side == PatternTableSide::Right,
            self.sprite_table_side == PatternTableSide::Right,
            self.current_address_increment == AddressIncrement::Down,
            self.base_name_table_position as u8 & 0b0000_0010 != 0,
            self.base_name_table_position as u8 & 0b0000_0001 != 0,
        ])
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum ExtPinRole {
    Read,
    Write,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum SpriteHeight {
    Normal = 8,
    Tall = 16,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum AddressIncrement {
    Right = 1,
    Down = 32,
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
