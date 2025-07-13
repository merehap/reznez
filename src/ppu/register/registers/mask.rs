// modular_bitfield pedantic clippy warnings
#![allow(clippy::cast_lossless, clippy::no_effect_underscore_binding, clippy::map_unwrap_or)]
#![allow(unused_parens)]

use log::info;
use modular_bitfield::bitfield;

#[bitfield]
#[derive(Clone, Copy, Debug)]
pub struct Mask {
    pub greyscale_enabled: bool,
    pub left_background_columns_enabled: bool,
    pub left_sprite_columns_enabled: bool,
    pub background_enabled: bool,
    pub sprites_enabled: bool,
    pub emphasize_red: bool,
    pub emphasize_green: bool,
    pub emphasize_blue: bool,
}

impl Mask {
    pub fn all_disabled() -> Mask {
        Mask::new()
    }

    pub fn full_screen_enabled() -> Mask {
        Mask::new()
            .with_left_sprite_columns_enabled(true)
            .with_left_background_columns_enabled(true)
    }

    pub fn emphasis_index(self) -> usize {
        ((self.emphasize_blue() as usize) << 2)
            | ((self.emphasize_green() as usize) << 1)
            | (self.emphasize_red() as usize)
    }

    pub fn set(&mut self, value: u8) {
        let old_mask = *self;
        *self = Mask::from_bytes([value]);

        log_change(old_mask.emphasize_blue(), self.emphasize_blue(), "Blue emphasis");
        log_change(old_mask.emphasize_green(), self.emphasize_green(), "Green emphasis");
        log_change(old_mask.emphasize_red(), self.emphasize_red(), "Red emphasis");
        log_change(old_mask.sprites_enabled(), self.sprites_enabled(), "Sprites");
        log_change(old_mask.background_enabled(), self.background_enabled(), "Background");

        log_change(
            old_mask.left_sprite_columns_enabled(),
            self.left_sprite_columns_enabled(),
            "Left sprite columns",
        );
        log_change(
            old_mask.left_background_columns_enabled(),
            self.left_background_columns_enabled(),
            "Left background columns",
        );
        log_change(old_mask.greyscale_enabled(), self.greyscale_enabled(), "Greyscale");
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
