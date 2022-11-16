use crate::ppu::palette::palette_table_index::PaletteTableIndex;
use crate::util::bit_util::get_bit;

#[derive(Clone, Copy, Debug)]
pub struct SpriteAttributes {
    flip_vertically: bool,
    flip_horizontally: bool,
    priority: Priority,
    palette_table_index: PaletteTableIndex,
}

impl SpriteAttributes {
    pub fn new() -> SpriteAttributes {
        SpriteAttributes {
            flip_vertically:   false,
            flip_horizontally: false,
            priority: Priority::InFront,
            palette_table_index: PaletteTableIndex::Zero,
        }
    }

    pub fn from_u8(value: u8) -> SpriteAttributes {
        let palette_table_index = match (get_bit(value, 6), get_bit(value, 7)) {
            (false, false) => PaletteTableIndex::Zero,
            (false, true ) => PaletteTableIndex::One,
            (true , false) => PaletteTableIndex::Two,
            (true , true ) => PaletteTableIndex::Three,
        };

        SpriteAttributes {
            flip_vertically:   get_bit(value, 0),
            flip_horizontally: get_bit(value, 1),
            priority:          get_bit(value, 2).into(),
            palette_table_index,
        }
    }

    pub fn flip_vertically(self) -> bool {
        self.flip_vertically
    }

    pub fn flip_horizontally(self) -> bool {
        self.flip_horizontally
    }

    pub fn priority(self) -> Priority {
        self.priority
    }

    pub fn palette_table_index(self) -> PaletteTableIndex {
        self.palette_table_index
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Priority {
    InFront,
    Behind,
}

impl From<bool> for Priority {
    fn from(value: bool) -> Self {
        if value {
            Priority::Behind
        } else {
            Priority::InFront
        }
    }
}
