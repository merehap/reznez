use crate::ppu::pixel_index::{PixelIndex, PixelColumn, PixelRow};
use crate::ppu::palette::rgb::Rgb;
use crate::ppu::palette::rgbt::Rgbt;
use crate::ppu::render::ppm::Ppm;
use crate::ppu::sprite::Priority;

pub struct Frame {
    buffer: Box<[[Rgbt; Frame::WIDTH]; Frame::HEIGHT]>,
    sprite_buffer: Box<[[(Rgbt, Priority, bool); Frame::WIDTH]; Frame::HEIGHT]>,
    universal_background_rgb: Rgb,
}

impl Frame {
    pub const WIDTH: usize = 256;
    pub const HEIGHT: usize = 240;

    pub fn new() -> Frame {
        Frame {
            buffer: Box::new([[Rgbt::Transparent; Frame::WIDTH]; Frame::HEIGHT]),
            sprite_buffer: Box::new([[(Rgbt::Transparent, Priority::Behind, false); Frame::WIDTH]; Frame::HEIGHT]),
            universal_background_rgb: Rgb::BLACK,
        }
    }

    pub fn pixel(&self, column: PixelColumn, row: PixelRow) -> (Rgb, Sprite0Hit) {
        let background_pixel = self.buffer[row.to_usize()][column.to_usize()];
        let (sprite_pixel, sprite_priority, is_sprite_0) =
            self.sprite_buffer[row.to_usize()][column.to_usize()];

        use Sprite0Hit::{Hit, Miss};
        let sprite_0_hit = if is_sprite_0 {Hit} else {Miss};

        use Rgbt::{Transparent, Opaque};
        match (background_pixel, sprite_pixel, sprite_priority) {
            (Transparent, Transparent, _) => (self.universal_background_rgb, Miss),
            (Transparent, Opaque(rgb), _) => (rgb, Miss),
            (Opaque(rgb), Transparent, _) => (rgb, Miss),
            (Opaque(_)  , Opaque(rgb), Priority::InFront) => (rgb, sprite_0_hit),
            (Opaque(rgb), Opaque(_  ), Priority::Behind) => (rgb, sprite_0_hit),
        }
    }

    pub fn set_universal_background_rgb(&mut self, rgb: Rgb) {
        self.universal_background_rgb = rgb;
    }

    pub fn clear_background_buffer(&mut self) {
        self.buffer = Box::new([[Rgbt::Transparent; Frame::WIDTH]; Frame::HEIGHT]);
    }

    pub fn clear_sprite_buffer(&mut self) {
        self.sprite_buffer = Box::new([[(Rgbt::Transparent, Priority::Behind, false); Frame::WIDTH]; Frame::HEIGHT]);
    }

    #[inline]
    pub fn background_row(&mut self, row: u8) -> &mut [Rgbt; Frame::WIDTH] {
        &mut self.buffer[row as usize]
    }

    #[inline]
    pub fn set_sprite_pixel(
        &mut self,
        column: PixelColumn,
        row: PixelRow,
        rgb: Rgb,
        priority: Priority,
        is_sprite_0: bool,
    ) {
        self.sprite_buffer[row.to_usize()][column.to_usize()] =
            (Rgbt::Opaque(rgb), priority, is_sprite_0);
    }

    pub fn write_all_pixel_data(
        &self,
        mut data: [u8; 3 * PixelIndex::PIXEL_COUNT],
    ) -> [u8; 3 * PixelIndex::PIXEL_COUNT] {

        for pixel_index in PixelIndex::iter() {
            let (column, row) = pixel_index.to_column_row();
            let (pixel, _) = self.pixel(column, row);

            let index = 3 * pixel_index.to_usize();
            data[index]     = pixel.red();
            data[index + 1] = pixel.green();
            data[index + 2] = pixel.blue();
        }

        data
    }

    pub fn to_ppm(&self) -> Ppm {
        let mut data = [0; 3 * Frame::WIDTH * Frame::HEIGHT];
        data = self.write_all_pixel_data(data);
        Ppm::new(data.to_vec())
    }
}

#[derive(Clone, Copy)]
pub enum Sprite0Hit {
    Hit,
    Miss,
}

impl Sprite0Hit {
    pub fn hit(self) -> bool {
        matches!(self, Sprite0Hit::Hit)
    }
}
