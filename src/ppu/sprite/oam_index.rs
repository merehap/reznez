#[derive(Clone, Copy, Debug)]
pub struct OamIndex {
    // "n" in the documentation
    sprite_index: u8,
    // "m" in the documentation
    field_index: FieldIndex,
    // Buggy sprite overflow offset.
    sprite_start_field_index: FieldIndex,
}

impl OamIndex {
    const MAX_SPRITE_INDEX: u8 = 63;

    pub fn new() -> OamIndex {
        OamIndex {
            sprite_index: 0,
            field_index: FieldIndex::YCoordinate,
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
        *self = OamIndex::new();
    }

    pub fn next_sprite(&mut self) -> bool {
        let end_reached = self.sprite_index == OamIndex::MAX_SPRITE_INDEX;
        if end_reached {
            self.sprite_index = 0;
        } else {
            self.sprite_index += 1;
        }

        end_reached
    }

    pub fn next_field(&mut self) -> bool {
        self.field_index.increment();
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
    }

    pub fn to_u8(self) -> u8 {
        (self.sprite_index << 2) | self.field_index as u8
    }
}

#[derive(PartialEq, Clone, Copy, Debug)]
enum FieldIndex {
    YCoordinate  = 0,
    PatternIndex = 1,
    Attributes   = 2,
    XCoordinate  = 3,
}

impl FieldIndex {
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
