use crate::util::get_bit;

#[derive(Clone, Copy, Debug)]
pub struct Sprite {
    x_coordinate: u8,
    y_coordinate: u8,
    tile_number: u8,
    flip_vertically: bool,
    flip_horizontally: bool,
    priority: Priority,
    palette_low: bool,
    palette_high: bool,
}

impl Sprite {
    pub fn from_u32(value: u32) -> Sprite {
        let [y_coordinate, tile_number, attribute, x_coordinate] =
            value.to_be_bytes();
        Sprite {
            x_coordinate,
            y_coordinate,
            tile_number,
            flip_vertically:   get_bit(attribute, 0),
            flip_horizontally: get_bit(attribute, 1),
            priority:          get_bit(attribute, 2).into(),
            palette_low:       get_bit(attribute, 6),
            palette_high:      get_bit(attribute, 7),
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

    pub fn palette_low(self) -> bool {
        self.palette_low
    }

    pub fn palette_high(self) -> bool {
        self.palette_high
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Priority {
    InFront,
    Behind,
}

impl Into<Priority> for bool {
    fn into(self) -> Priority {
        if self {
            Priority::InFront
        } else {
            Priority::Behind
        }
    }
}
