use crate::util::get_bit;

#[derive(Clone, Copy)]
pub struct Mask {
    emphasize_blue: bool,
    emphasize_green: bool,
    emphasize_red: bool,
    sprites_enabled: bool,
    background_enabled: bool,
    left_column_sprites_enabled: bool,
    left_column_background_enabled: bool,
    greyscale_enabled: bool,
}

impl Mask {
    pub fn new() -> Mask {
        Mask {
            emphasize_blue: false,
            emphasize_green: false,
            emphasize_red: false,
            sprites_enabled: false,
            background_enabled: false,
            left_column_sprites_enabled: false,
            left_column_background_enabled: false,
            greyscale_enabled: false,
        }
    }

    pub fn from_u8(value: u8) -> Mask {
        Mask {
            emphasize_blue:                 get_bit(value, 0),
            emphasize_green:                get_bit(value, 1),
            emphasize_red:                  get_bit(value, 2),
            sprites_enabled:                get_bit(value, 3),
            background_enabled:             get_bit(value, 4),
            left_column_sprites_enabled:    get_bit(value, 5),
            left_column_background_enabled: get_bit(value, 6),
            greyscale_enabled:              get_bit(value, 7),
        }
    }

    pub fn emphasize_blue(self) -> bool {
        self.emphasize_blue
    }

    pub fn emphasize_green(self) -> bool {
        self.emphasize_green
    }

    pub fn emphasize_red(self) -> bool {
        self.emphasize_red
    }

    pub fn sprites_enabled(self) -> bool {
        self.sprites_enabled
    }

    pub fn background_enabled(self) -> bool {
        self.background_enabled
    }

    pub fn left_column_sprites_enabled(self) -> bool {
        self.left_column_sprites_enabled
    }

    pub fn left_column_background_enabled(self) -> bool {
        self.left_column_background_enabled
    }

    pub fn greyscale_enabled(self) -> bool {
        self.greyscale_enabled
    }
}
