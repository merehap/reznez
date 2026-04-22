use splitbits::splitbits;

use crate::ppu::palette::palette_table_index::PaletteTableIndex;

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
        let fields = splitbits!(value, "vhp. ..ii");
        let palette_table_index = match fields.i {
            0 => PaletteTableIndex::Zero,
            1 => PaletteTableIndex::One,
            2 => PaletteTableIndex::Two,
            3 => PaletteTableIndex::Three,
            _ => unreachable!(),
        };

        SpriteAttributes {
            flip_vertically:   fields.v,
            flip_horizontally: fields.h,
            priority:          fields.p.into(),
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
