use log::info;
use ux::u2;

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct OamAddress {
    addr: u8,
    // Buggy sprite overflow offset.
    sprite_start_field_index: u2,
}

impl OamAddress {
    const MAX_SPRITE_INDEX: u8 = 63;

    pub fn from_u8(value: u8) -> OamAddress {
        OamAddress {
            addr: value,
            // This field keeps its initial value unless a sprite overflow occurs.
            sprite_start_field_index: u2::new(0),
        }
    }

    pub fn new_sprite_started(self) -> bool {
        u2::new(self.addr % 4) == self.sprite_start_field_index
    }

    pub fn is_at_sprite_0(self) -> bool {
        self.addr < 4
    }

    pub fn reset(&mut self) {
        info!(target: "oamaddr", "\tResetting OamAddress to 0x00.");
        self.addr = 0;
        self.sprite_start_field_index = u2::new(0);
    }

    pub fn increment(&mut self) {
        self.addr = self.addr.wrapping_add(1);
        info!(target: "oamaddr", "\tIncrementing OamAddress to 0x{:02X}.", self.to_u8());
    }

    pub fn next_sprite(&mut self) -> bool {
        let end_reached = self.addr / 4 == OamAddress::MAX_SPRITE_INDEX;
        if end_reached {
            self.addr %= 4;
        } else {
            self.addr += 4;
        }

        info!(target: "oamaddr", "\tAdvancing to next sprite OamAddress 0x{:02X}.", self.to_u8());

        end_reached
    }

    pub fn next_field(&mut self) -> bool {
        self.addr = self.addr.wrapping_add(1);
        if self.addr % 4 == 0 {
            self.addr -= 4;
        }

        info!(target: "oamaddr", "\tAdvancing to next field OamAddress 0x{:02X}.", self.to_u8());
        let carry = self.addr % 4 == 0;
        if carry {
            self.next_sprite()
        } else {
            false
        }
    }

    pub fn corrupt_sprite_y_index(&mut self) {
        self.addr = self.addr.wrapping_add(1);
        if self.addr % 4 == 0 {
            self.addr -= 4;
        }

        self.sprite_start_field_index = u2::new(self.addr % 4);
        info!(target: "oamaddr", "\tCorrupting OamAddress 0x{:02X}.", self.to_u8());
    }

    pub fn to_u8(self) -> u8 {
        self.addr
    }
}