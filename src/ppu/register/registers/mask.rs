use crate::util::bit_util::get_bit;

#[derive(Clone, Copy, Debug)]
pub struct Mask {
    pub emphasize_blue: bool,
    pub emphasize_green: bool,
    pub emphasize_red: bool,
    pub sprites_enabled: bool,
    pub background_enabled: bool,
    pub left_sprite_columns_enabled: bool,
    pub left_background_columns_enabled: bool,
    pub greyscale_enabled: bool,
}

impl Mask {
    pub fn new() -> Mask {
        Mask {
            emphasize_blue: false,
            emphasize_green: false,
            emphasize_red: false,
            sprites_enabled: false,
            background_enabled: false,
            left_sprite_columns_enabled: false,
            left_background_columns_enabled: false,
            greyscale_enabled: false,
        }
    }

    pub fn from_u8(value: u8) -> Mask {
        Mask {
            emphasize_blue:                  get_bit(value, 0),
            emphasize_green:                 get_bit(value, 1),
            emphasize_red:                   get_bit(value, 2),
            sprites_enabled:                 get_bit(value, 3),
            background_enabled:              get_bit(value, 4),
            left_sprite_columns_enabled:     get_bit(value, 5),
            left_background_columns_enabled: get_bit(value, 6),
            greyscale_enabled:               get_bit(value, 7),
        }
    }
}
