use log::info;
use splitbits::splitbits;

#[derive(Clone, Copy, Debug, Default)]
pub struct Mask {
    greyscale_enabled: bool,
    left_background_columns_enabled: bool,
    left_sprite_columns_enabled: bool,
    background_enabled: bool,
    sprites_enabled: bool,
    emphasize_red: bool,
    emphasize_green: bool,
    emphasize_blue: bool,
}

impl Mask {
    pub fn all_disabled() -> Self {
        Self::default()
    }

    pub fn full_screen_enabled() -> Mask {
        Self {
            left_background_columns_enabled: true,
            left_sprite_columns_enabled: true,
            .. Self::all_disabled()
        }
    }

    pub fn emphasis_index(self) -> usize {
        ((self.emphasize_blue as usize) << 2)
            | ((self.emphasize_green as usize) << 1)
            | (self.emphasize_red as usize)
    }

    pub fn greyscale_enabled(&self) -> bool {
        self.greyscale_enabled
    }

    pub fn left_background_columns_enabled(&self) -> bool {
        self.left_background_columns_enabled
    }

    pub fn left_sprite_columns_enabled(&self) -> bool {
        self.left_sprite_columns_enabled
    }

    pub fn background_enabled(&self) -> bool {
        self.background_enabled
    }

    pub fn sprites_enabled(&self) -> bool {
        self.sprites_enabled
    }

    pub fn set(&mut self, value: u8) {
        let old_mask = *self;
        let fields = splitbits!(value, "zlmb sefg");
        *self = Self {
            emphasize_blue: fields.z,
            emphasize_green: fields.l,
            emphasize_red: fields.m,
            sprites_enabled: fields.b,
            background_enabled: fields.s,
            left_sprite_columns_enabled: fields.e,
            left_background_columns_enabled: fields.f,
            greyscale_enabled: fields.g,
        };

        log_change(old_mask.emphasize_blue, self.emphasize_blue, "Blue emphasis");
        log_change(old_mask.emphasize_green, self.emphasize_green, "Green emphasis");
        log_change(old_mask.emphasize_red, self.emphasize_red, "Red emphasis");
        log_change(old_mask.sprites_enabled, self.sprites_enabled, "Sprites");
        log_change(old_mask.background_enabled, self.background_enabled, "Background");

        log_change(
            old_mask.left_sprite_columns_enabled,
            self.left_sprite_columns_enabled,
            "Left sprite columns",
        );
        log_change(
            old_mask.left_background_columns_enabled,
            self.left_background_columns_enabled,
            "Left background columns",
        );
        log_change(old_mask.greyscale_enabled, self.greyscale_enabled, "Greyscale");
    }
}

fn log_change(old: bool, new: bool, message_prefix: &str) {
    let message = match (old, new) {
        (false, true) => format!("\t{message_prefix} enabled."),
        (true, false) => format!("\t{message_prefix} disabled."),
        _ => return,
    };
    info!(target: "ppuflags", "{message}");
}
