use log::info;

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct OamAddress(u8);

impl OamAddress {
    pub fn from_u8(value: u8) -> OamAddress {
        info!(target: "oamaddr", "\tSetting OamAddress to 0x{value:02X}.");
        OamAddress(value)
    }

    pub fn reset(&mut self) {
        info!(target: "oamaddr", "\tResetting OamAddress to 0x00.");
        self.0 = 0;
    }

    pub fn next_sprite(&mut self) -> bool {
        let end_reached;
        (self.0, end_reached) = self.0.overflowing_add(4);
        info!(target: "oamaddr", "\tAdvancing to next sprite OamAddress 0x{:02X}.", self.0);
        end_reached
    }

    pub fn next_field(&mut self) -> bool {
        let end_reached;
        (self.0, end_reached) = self.0.overflowing_add(1);
        info!(target: "oamaddr", "\tAdvancing to next field OamAddress 0x{:02X}.", self.0);
        end_reached
    }

    pub fn corrupt_sprite_y_index(&mut self) {
        self.0 = self.0.wrapping_add(1);
        if self.0 % 4 == 0 {
            self.0 -= 4;
        }

        info!(target: "oamaddr", "\tCorrupting OamAddress 0x{:02X}.", self.0);
    }

    // Suspiciously similar to corrupt_sprite_y_index()
    pub fn corrupt_by_write(&mut self) {
        self.0 = self.0.wrapping_add(4) & 0xFC;
    }

    pub fn to_u8(self) -> u8 {
        self.0
    }
}