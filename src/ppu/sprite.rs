#[derive(Clone, Copy, Debug)]
pub struct Sprite {
    x_coordinate: u8,
    y_coordinate: u8,
    tile_number: u8,
    attribute: u8,
}

impl Sprite {
    pub fn from_u32(value: u32) -> Sprite {
        Sprite {
            y_coordinate: (value >> 24) as u8,
            tile_number: (value >> 16) as u8,
            attribute: (value >> 8) as u8,
            x_coordinate: value as u8,
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

    pub fn attribute(self) -> u8 {
        self.attribute
    }
}
