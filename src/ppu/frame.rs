use crate::ppu::palette::rgb::Rgb;
use crate::ppu::palette::rgbt::Rgbt;
use crate::ppu::sprite::Priority;

type SpriteSliver = ([Rgbt; 8], Priority);

pub struct Frame {
    buffer: [[Rgbt; Frame::WIDTH]; Frame::HEIGHT],
    sprite_buffer: [[SpriteSliver; Frame::WIDTH / 8]; Frame::HEIGHT],
    universal_background_rgb: Rgb,
}

impl Frame {
    pub const WIDTH: usize = 256;
    pub const HEIGHT: usize = 240;

    pub fn new() -> Frame {
        Frame {
            buffer: [[Rgbt::Transparent; Frame::WIDTH]; Frame::HEIGHT],
            sprite_buffer: new_sprite_buffer(),
            universal_background_rgb: Rgb::BLACK,
        }
    }

    pub fn pixel(&self, column: u8, row: u8) -> Rgb {
        let row = row as usize;
        let column = column as usize;
        let background_pixel = self.buffer[row][column];
        let (sprite_sliver, sprite_priority) = self.sprite_buffer[row][column / 8];
        let sprite_pixel = sprite_sliver[column % 8];

        use Rgbt::*;
        match (background_pixel, sprite_pixel, sprite_priority) {
            (Transparent, Transparent, _) => self.universal_background_rgb,
            (Transparent, Opaque(rgb), _) => rgb,
            (Opaque(rgb), Transparent, _) => rgb,
            (Opaque(_)  , Opaque(rgb), Priority::InFront) => rgb,
            (Opaque(rgb), Opaque(_  ), Priority::Behind) => rgb,
        }
    }

    pub fn set_universal_background_rgb(&mut self, rgb: Rgb) {
        self.universal_background_rgb = rgb;
    }

    pub fn clear_sprite_buffer(&mut self) {
        self.sprite_buffer = new_sprite_buffer();
    }

    #[inline]
    pub fn background_tile_sliver(&mut self, column: u8, row: u8) -> &mut [Rgbt; 8] {
        let row_slice = &mut self.buffer[row as usize];
        let column_slice = &mut row_slice[column as usize..column as usize + 8];
        column_slice
            .try_into()
            .unwrap()
    }

    #[inline]
    pub fn sprites_tile_sliver(
        &mut self,
        column: u8,
        row: u8,
        ) -> &mut ([Rgbt; 8], Priority) {

        let row_slice = &mut self.sprite_buffer[row as usize];
        &mut row_slice[(column / 8) as usize]
    }
}

fn new_sprite_buffer() -> [[SpriteSliver; Frame::WIDTH / 8]; Frame::HEIGHT] {
    [[([Rgbt::Transparent; 8], Priority::Behind); Frame::WIDTH / 8]; Frame::HEIGHT]
}
