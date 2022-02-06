use crate::ppu::name_table::name_table_number::NameTableNumber;
use crate::ppu::pattern_table::PatternTableSide;
use crate::util::bit_util::{get_bit, pack_bools};

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct Ctrl {
    pub nmi_enabled: bool,
    pub ext_pin_role: ExtPinRole,
    pub sprite_width: SpriteWidth,
    pub background_table_side: PatternTableSide,
    pub sprite_table_side: PatternTableSide,
    pub vram_address_increment: VramAddressIncrement,
    pub name_table_number: NameTableNumber,
}

impl Ctrl {
    pub fn new() -> Ctrl {
        Ctrl {
            nmi_enabled: false,
            ext_pin_role: ExtPinRole::Read,
            sprite_width: SpriteWidth::Normal,
            background_table_side: PatternTableSide::Left,
            sprite_table_side: PatternTableSide::Left,
            vram_address_increment: VramAddressIncrement::Right,
            name_table_number: NameTableNumber::Zero,
        }
    }

    pub fn from_u8(value: u8) -> Ctrl {
        Ctrl {
            nmi_enabled: get_bit(value, 0),
            ext_pin_role:
                if get_bit(value, 1) {
                    ExtPinRole::Write
                } else {
                    ExtPinRole::Read
                },
            sprite_width:
                if get_bit(value, 2) {
                    todo!("Wide sprites are not supported yet.");
                    //SpriteWidth::Wide
                } else {
                    SpriteWidth::Normal
                },
            background_table_side:
                if get_bit(value, 3) {
                    PatternTableSide::Right
                } else {
                    PatternTableSide::Left
                },
            sprite_table_side:
                if get_bit(value, 4) {
                    PatternTableSide::Right
                } else {
                    PatternTableSide::Left
                },
            vram_address_increment:
                if get_bit(value, 5) {
                    VramAddressIncrement::Down
                } else {
                    VramAddressIncrement::Right
                },
            name_table_number:
                match (get_bit(value, 6), get_bit(value, 7)) {
                    (false, false) => NameTableNumber::Zero,
                    (false, true ) => NameTableNumber::One,
                    (true , false) => NameTableNumber::Two,
                    (true , true ) => NameTableNumber::Three,
                },
        }
    }

    #[allow(dead_code)]
    pub fn to_u8(self) -> u8 {
        pack_bools(
            [
                self.nmi_enabled,
                self.ext_pin_role == ExtPinRole::Write,
                self.sprite_width == SpriteWidth::Wide,
                self.background_table_side == PatternTableSide::Right,
                self.sprite_table_side == PatternTableSide::Right,
                self.vram_address_increment == VramAddressIncrement::Down,
                self.name_table_number as u8 & 0b0000_0010 != 0,
                self.name_table_number as u8 & 0b0000_0001 != 0,
            ]
        )
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum ExtPinRole {
    Read,
    Write,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum SpriteWidth {
    Normal = 8,
    Wide = 16,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum VramAddressIncrement {
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
