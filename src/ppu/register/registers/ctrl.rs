use splitbits::splitbits;

use crate::ppu::name_table::name_table_quadrant::NameTableQuadrant;
use crate::ppu::pattern_table_side::PatternTableSide;
use crate::ppu::sprite::sprite_height::SpriteHeight;

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct Ctrl {
    pub nmi_enabled: bool,
    #[allow(dead_code)]
    pub ext_pin_role: ExtPinRole,
    pub sprite_height: SpriteHeight,
    pub background_table_side: PatternTableSide,
    pub sprite_table_side: PatternTableSide,
    pub current_address_increment: AddressIncrement,
    pub base_name_table_quadrant: NameTableQuadrant,
}

impl Ctrl {
    pub fn new() -> Self {
        Self {
            nmi_enabled: false,
            #[allow(dead_code)]
            ext_pin_role: ExtPinRole::Read,
            sprite_height: SpriteHeight::Normal,
            background_table_side: PatternTableSide::Left,
            sprite_table_side: PatternTableSide::Left,
            current_address_increment: AddressIncrement::Right,
            base_name_table_quadrant: NameTableQuadrant::TopLeft,
        }
    }

    pub fn from_u8(value: u8) -> Ctrl {
        let fields = splitbits!(value, "nehbsiqq");
        Self {
            nmi_enabled: fields.n,
            ext_pin_role: [ExtPinRole::Read, ExtPinRole::Write][fields.e as usize],
            sprite_height: [SpriteHeight::Normal, SpriteHeight::Tall][fields.h as usize],
            background_table_side: [PatternTableSide::Left, PatternTableSide::Right][fields.b as usize],
            sprite_table_side: [PatternTableSide::Left, PatternTableSide::Right][fields.s as usize],
            current_address_increment: [AddressIncrement::Right, AddressIncrement::Down][fields.i as usize],
            base_name_table_quadrant: NameTableQuadrant::ALL[fields.q as usize],
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum ExtPinRole {
    Read,
    Write,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum AddressIncrement {
    Right,
    Down,
}