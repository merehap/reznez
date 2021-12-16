use crate::ppu::palette::palette_table_index::PaletteTableIndex;
use crate::util::get_bit;

#[derive(Clone, Copy, Debug)]
pub struct Sprite {
    x_coordinate: u8,
    y_coordinate: u8,
    tile_number: u8,
    flip_vertically: bool,
    flip_horizontally: bool,
    priority: Priority,
    palette_table_index: PaletteTableIndex,
}

impl Sprite {
    pub fn from_u32(value: u32) -> Sprite {
        let [y_coordinate, tile_number, attribute, x_coordinate] =
            value.to_be_bytes();

        let palette_table_index =
            match (get_bit(attribute, 6), get_bit(attribute, 7)) {
                (false, false) => PaletteTableIndex::Zero,
                (true , false) => PaletteTableIndex::One,
                (false, true ) => PaletteTableIndex::Two,
                (true , true ) => PaletteTableIndex::Three,
            };

        Sprite {
            x_coordinate,
            y_coordinate,
            tile_number,
            flip_vertically:   get_bit(attribute, 0),
            flip_horizontally: get_bit(attribute, 1),
            priority:          get_bit(attribute, 2).into(),
            palette_table_index,
        }
    }

    pub fn x_coordinate(self) -> u8 {
        self.x_coordinate
    }

    pub fn y_coordinate(self) -> u8 {
        self.y_coordinate
    }

    pub fn tile_number(self) -> u8 {
        self.tile_number
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

#[derive(Clone, Copy, Debug)]
pub enum Priority {
    InFront,
    Behind,
}

impl From<bool> for Priority {
    fn from(value: bool) -> Self {
        if value {
            Priority::InFront
        } else {
            Priority::Behind
        }
    }
}
