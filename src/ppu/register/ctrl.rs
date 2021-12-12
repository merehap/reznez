use crate::ppu::name_table_number::NameTableNumber;
use crate::ppu::pattern_table::PatternTableSide;
use crate::util::get_bit;

#[derive(Clone, Copy, Debug)]
pub struct Ctrl {
    vblank_nmi: VBlankNmi,
    ext_pin_role: ExtPinRole,
    sprite_width: SpriteWidth,
    background_table_side: PatternTableSide,
    sprite_table_side: PatternTableSide,
    vram_address_increment: VramAddressIncrement,
    name_table_number: NameTableNumber,
}

impl Ctrl {
    pub fn new() -> Ctrl {
        Ctrl {
            vblank_nmi: VBlankNmi::Off,
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
            vblank_nmi:
                if get_bit(value, 0) {
                    VBlankNmi::On
                } else {
                    VBlankNmi::Off
                },
            ext_pin_role:
                if get_bit(value, 1) {
                    ExtPinRole::Write
                } else {
                    ExtPinRole::Read
                },
            sprite_width:
                if get_bit(value, 2) {
                    SpriteWidth::Wide
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

    pub fn vblank_nmi(self) -> VBlankNmi {
        self.vblank_nmi
    }

    pub fn ext_pin_role(self) -> ExtPinRole {
        self.ext_pin_role
    }

    pub fn sprite_width(self) -> SpriteWidth {
        self.sprite_width
    }

    pub fn background_table_side(self) -> PatternTableSide {
        self.background_table_side
    }

    pub fn sprite_table_side(self) -> PatternTableSide {
        self.sprite_table_side
    }

    pub fn vram_address_increment(self) -> VramAddressIncrement {
        self.vram_address_increment
    }

    pub fn name_table_number(self) -> NameTableNumber {
        self.name_table_number
    }
}

#[derive(Clone, Copy, Debug)]
pub enum VBlankNmi {
    Off,
    On,
}

#[derive(Clone, Copy, Debug)]
pub enum ExtPinRole {
    Read,
    Write,
}

#[derive(Clone, Copy, Debug)]
pub enum SpriteWidth {
    Normal = 8,
    Wide = 16,
}

#[derive(Clone, Copy, Debug)]
pub enum VramAddressIncrement {
    Right = 1,
    Down = 32,
}
