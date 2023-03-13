use log::info;

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct OamAddress {
    // "n" in the documentation
    sprite_index: u8,
    // "m" in the documentation
    field_index: FieldIndex,
    // Buggy sprite overflow offset.
    sprite_start_field_index: FieldIndex,
}

impl OamAddress {
    const MAX_SPRITE_INDEX: u8 = 63;

    pub fn new() -> OamAddress {
        OamAddress {
            sprite_index: 0,
            field_index: FieldIndex::YCoordinate,
            // This field keeps its initial value unless a sprite overflow occurs.
            sprite_start_field_index: FieldIndex::YCoordinate,
        }
    }

    pub fn from_u8(value: u8) -> OamAddress {
        OamAddress {
            sprite_index: value >> 2,
            field_index: FieldIndex::from_u8(value & 0b11),
            // This field keeps its initial value unless a sprite overflow occurs.
            sprite_start_field_index: FieldIndex::YCoordinate,
        }
    }

    pub fn new_sprite_started(self) -> bool {
        self.field_index == self.sprite_start_field_index
    }

    pub fn is_at_sprite_0(self) -> bool {
        self.sprite_index == 0
    }

    pub fn reset(&mut self) {
        info!(target: "oamaddr", "\tResetting OamAddress to 0x00.");
        *self = OamAddress::new();
    }

    pub fn increment(&mut self) {
        // TODO: Make efficient?
        *self = OamAddress::from_u8(self.to_u8().wrapping_add(1));
        info!(target: "oamaddr", "\tIncrementing OamAddress to 0x{:02X}.", self.to_u8());
    }

    pub fn next_sprite(&mut self) -> bool {
        let end_reached = self.sprite_index == OamAddress::MAX_SPRITE_INDEX;
        if end_reached {
            self.sprite_index = 0;
        } else {
            self.sprite_index += 1;
        }

        info!(target: "oamaddr", "\tAdvancing to next sprite OamAddress 0x{:02X}.", self.to_u8());

        end_reached
    }

    pub fn next_field(&mut self) -> bool {
        self.field_index.increment();
        info!(target: "oamaddr", "\tAdvancing to next field OamAddress 0x{:02X}.", self.to_u8());
        let carry = self.field_index == FieldIndex::YCoordinate;
        if carry {
            self.next_sprite()
        } else {
            false
        }
    }

    pub fn corrupt_sprite_y_index(&mut self) {
        self.field_index.increment();
        self.sprite_start_field_index = self.field_index;
        info!(target: "oamaddr", "\tCorrupting OamAddress 0x{:02X}.", self.to_u8());
    }

    pub fn to_u8(self) -> u8 {
        (self.sprite_index << 2) | self.field_index as u8
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
enum FieldIndex {
    YCoordinate  = 0,
    PatternIndex = 1,
    Attributes   = 2,
    XCoordinate  = 3,
}

impl FieldIndex {
    pub fn from_u8(value: u8) -> FieldIndex {
        use FieldIndex::*;
        match value {
            0 => YCoordinate,
            1 => PatternIndex,
            2 => Attributes,
            3 => XCoordinate,
            _ => unreachable!(),
        }
    }

    pub fn increment(&mut self) {
        use FieldIndex::*;
        *self = match self {
            YCoordinate  => PatternIndex,
            PatternIndex => Attributes,
            Attributes   => XCoordinate,
            XCoordinate  => YCoordinate,
        };
    }
}
