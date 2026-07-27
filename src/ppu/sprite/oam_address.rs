use log::info;
use splitbits::{combinebits, splitbits_named_ux};
use ux::{u2, u6};

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct OamAddress {
    // "n" from https://www.nesdev.org/wiki/PPU_sprite_evaluation
    sprite: u6,
    // "m" from https://www.nesdev.org/wiki/PPU_sprite_evaluation
    field: u2,
    all_sprites_evaluated: bool,
}

impl OamAddress {
    pub fn new() -> Self {
        Self {
            sprite: u6::MIN,
            field: u2::MIN,
            all_sprites_evaluated: false,
        }
    }

    pub fn from_u8(addr: u8) -> Self {
        info!(target: "oamaddr", "\tSetting OamAddress to 0x{addr:02X}.");
        let (sprite, field) = splitbits_named_ux!(addr, "ssss ssff");
        Self {
            sprite,
            field,
            all_sprites_evaluated: false,
        }
    }

    pub fn all_sprites_evaluated(self) -> bool {
        self.all_sprites_evaluated
    }

    pub fn clear_all_sprites_evaluated(&mut self) {
        self.all_sprites_evaluated = false;
    }

    pub fn reset(&mut self) {
        info!(target: "oamaddr", "\tResetting OamAddress to 0x00.");
        self.sprite = u6::MIN;
        self.field = u2::MIN;
    }

    pub fn next_sprite(&mut self) {
        self.sprite = self.sprite.wrapping_add(u6::new(1));
        if self.sprite == u6::MIN {
            self.all_sprites_evaluated = true;
        }

        info!(target: "oamaddr", "\tAdvancing to next sprite OamAddress 0x{:02X}.", self.to_u8());
    }

    pub fn next_field(&mut self) -> bool {
        self.field = self.field.wrapping_add(u2::new(1));
        let overflowed = self.field == u2::MIN;
        if overflowed {
            self.next_sprite();
        }

        info!(target: "oamaddr", "\tAdvancing to next field OamAddress 0x{:02X}.", self.to_u8());
        overflowed
    }

    pub fn corrupt_sprite_y_index(&mut self) {
        self.next_sprite();
        self.field = self.field.wrapping_add(u2::new(1));
        info!(target: "oamaddr", "\tCorrupting OamAddress by sprite overflow 0x{:02X}.", self.to_u8());
    }

    pub fn corrupt_by_write(&mut self) {
        self.next_sprite();
        self.field = u2::MIN;
        info!(target: "oamaddr", "\tCorrupting OamAddress by write 0x{:02X}.", self.to_u8());
    }

    pub fn to_u8(self) -> u8 {
        combinebits!(self.sprite, self.field, "ssss ssff")
    }
}