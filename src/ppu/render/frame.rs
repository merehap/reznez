use std::ops::{Index, IndexMut};

use enum_iterator::IntoEnumIterator;

use crate::ppu::pixel_index::{PixelIndex, PixelColumn, ColumnInTile, PixelRow, RowInTile};
use crate::ppu::palette::rgb::Rgb;
use crate::ppu::palette::rgbt::Rgbt;
use crate::ppu::pattern_table::Tile;
use crate::ppu::register::registers::mask::Mask;
use crate::ppu::render::ppm::Ppm;
use crate::ppu::sprite::Priority;

pub struct Frame {
    buffer: FrameBuffer<Rgbt>,
    sprite_buffer: FrameBuffer<(Rgbt, Priority, bool)>,
    universal_background_rgb: Rgb,
}

impl Frame {
    pub fn new() -> Frame {
        Frame {
            buffer: FrameBuffer::filled(Rgbt::Transparent),
            sprite_buffer: FrameBuffer::filled((Rgbt::Transparent, Priority::Behind, false)),
            universal_background_rgb: Rgb::BLACK,
        }
    }

    pub fn pixel(&self, mask: Mask, column: PixelColumn, row: PixelRow) -> (Rgb, Sprite0Hit) {
        use Rgbt::{Transparent, Opaque};
        let mut background_pixel = self.buffer[(column, row)];
        if !mask.left_background_columns_enabled && column.is_in_left_margin() {
            background_pixel = Transparent;
        }

        let (mut sprite_pixel, sprite_priority, is_sprite_0) =
            self.sprite_buffer[(column, row)];
        if !mask.left_sprite_columns_enabled && column.is_in_left_margin() {
            sprite_pixel = Transparent;
        }

        use Sprite0Hit::{Hit, Miss};
        let sprite_0_hit = if is_sprite_0 {Hit} else {Miss};

        // https://wiki.nesdev.org/w/index.php?title=PPU_OAM#Sprite_zero_hits
        use Priority::{InFront, Behind};
        match (background_pixel, sprite_pixel, sprite_priority, column) {
            (Transparent, Transparent, _, _) => (self.universal_background_rgb, Miss),
            (Transparent, Opaque(rgb), _, _) => (rgb, Miss),
            (Opaque(rgb), Transparent, _, _) => (rgb, Miss),
            (Opaque(_)  , Opaque(rgb), InFront, PixelColumn::MAX) => (rgb, Miss),
            (Opaque(rgb), Opaque(_)  , Behind , PixelColumn::MAX) => (rgb, Miss),
            (Opaque(_)  , Opaque(rgb), InFront, _) => (rgb, sprite_0_hit),
            (Opaque(rgb), Opaque(_  ), Behind , _) => (rgb, sprite_0_hit),
        }
    }

    pub fn set_universal_background_rgb(&mut self, rgb: Rgb) {
        self.universal_background_rgb = rgb;
    }

    pub fn clear_sprite_line(&mut self, row: PixelRow) {
        for column in PixelColumn::iter() {
            self.sprite_buffer[(column, row)] =
                (Rgbt::Transparent, Priority::Behind, false);
        }
    }

    #[inline]
    pub fn set_background_pixel(
        &mut self,
        pixel_column: PixelColumn,
        pixel_row: PixelRow,
        rgbt: Rgbt,
    ) {
        self.buffer[(pixel_column, pixel_row)] = rgbt;
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
        self.sprite_buffer[(column, row)] =
            (Rgbt::Opaque(rgb), priority, is_sprite_0);
    }

    pub fn write_all_pixel_data(
        &self,
        mask: Mask,
        mut data: [u8; 3 * PixelIndex::PIXEL_COUNT],
    ) -> [u8; 3 * PixelIndex::PIXEL_COUNT] {

        for pixel_index in PixelIndex::iter() {
            let (column, row) = pixel_index.to_column_row();
            let (pixel, _) = self.pixel(mask, column, row);

            let index = 3 * pixel_index.to_usize();
            data[index]     = pixel.red();
            data[index + 1] = pixel.green();
            data[index + 2] = pixel.blue();
        }

        data
    }

    pub fn update_pixel_data(
        &self,
        mask: Mask,
        data: &mut [u8; 3 * PixelIndex::PIXEL_COUNT],
    ) {
        for pixel_index in PixelIndex::iter() {
            let (column, row) = pixel_index.to_column_row();
            let (pixel, _) = self.pixel(mask, column, row);

            let index = 3 * pixel_index.to_usize();
            data[index]     = pixel.red();
            data[index + 1] = pixel.green();
            data[index + 2] = pixel.blue();
        }
    }

    pub fn copy_to_rgba_buffer(
        &self,
        mask: Mask,
        buffer: &mut [u8; 4 * PixelIndex::PIXEL_COUNT],
    ) {
        for pixel_index in PixelIndex::iter() {
            let (column, row) = pixel_index.to_column_row();
            let (pixel, _) = self.pixel(mask, column, row);

            let index = 4 * pixel_index.to_usize();
            buffer[index]     = pixel.red();
            buffer[index + 1] = pixel.green();
            buffer[index + 2] = pixel.blue();
            // No transparency.
            buffer[index + 3] = 0xFF;
        }
    }

    pub fn to_ppm(&self, mask: Mask) -> Ppm {
        let mut data = [0; 3 * PixelIndex::PIXEL_COUNT];
        data = self.write_all_pixel_data(mask, data);
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

struct FrameBuffer<T>(Box<[[T; PixelColumn::COLUMN_COUNT]; PixelRow::ROW_COUNT]>);

impl <T: Copy> FrameBuffer<T> {
    fn filled(value: T) -> FrameBuffer<T> {
        FrameBuffer(Box::new([[value; PixelColumn::COLUMN_COUNT]; PixelRow::ROW_COUNT]))
    }
}

impl <T> Index<(PixelColumn, PixelRow)> for FrameBuffer<T> {
    type Output = T;

    fn index(&self, (column, row): (PixelColumn, PixelRow)) -> &T {
        &self.0[row.to_usize()][column.to_usize()]
    }
}

impl <T> IndexMut<(PixelColumn, PixelRow)> for FrameBuffer<T> {
    fn index_mut(&mut self, (column, row): (PixelColumn, PixelRow)) -> &mut T {
        &mut self.0[row.to_usize()][column.to_usize()]
    }
}

pub struct DebugBuffer<const WIDTH: usize, const HEIGHT: usize>(Box<[[Rgb; WIDTH]; HEIGHT]>);

impl <const WIDTH: usize, const HEIGHT: usize> DebugBuffer<WIDTH, HEIGHT> {
    pub fn filled(value: Rgb) -> DebugBuffer<WIDTH, HEIGHT> {
        DebugBuffer(Box::new([[value; WIDTH]; HEIGHT]))
    }

    pub fn place_frame(&mut self, left_column: usize, top_row: usize, frame: &Frame) {
        let mask = Mask::full_screen_enabled();
        for pixel_index in PixelIndex::iter() {
            let (column, row) = pixel_index.to_column_row();
            let (pixel, _) = frame.pixel(mask, column, row);
            self[(left_column + column.to_usize(), top_row + row.to_usize())] = pixel;
        }
    }

    pub fn place_tile(&mut self, left_column: usize, top_row: usize, tile: &Tile) {
        for row_in_tile in RowInTile::into_enum_iter() {
            for column_in_tile in ColumnInTile::into_enum_iter() {
                let column_in_tile = column_in_tile as usize;
                let row_in_tile = row_in_tile as usize;
                if let Rgbt::Opaque(rgb) = tile.0[row_in_tile][column_in_tile] {
                    self[(left_column + column_in_tile, top_row + row_in_tile)] = rgb;
                }
            }
        }
    }

    pub fn copy_to_rgba_buffer(&self, buffer: &mut [u8]) {
        for row in 0..HEIGHT {
            for column in 0..WIDTH {
                let index = 4 * (WIDTH * row + column);
                let pixel = self[(column, row)];
                buffer[index]     = pixel.red();
                buffer[index + 1] = pixel.green();
                buffer[index + 2] = pixel.blue();
                // No transparency.
                buffer[index + 3] = 0xFF;
            }
        }
    }
}

impl <const WIDTH: usize, const HEIGHT: usize> Index<(usize, usize)> for DebugBuffer<WIDTH, HEIGHT> {
    type Output = Rgb;

    fn index(&self, (column, row): (usize, usize)) -> &Rgb {
        &self.0[row][column]
    }
}

impl <const WIDTH: usize, const HEIGHT: usize> IndexMut<(usize, usize)> for DebugBuffer<WIDTH, HEIGHT> {
    fn index_mut(&mut self, (column, row): (usize, usize)) -> &mut Rgb {
        &mut self.0[row][column]
    }
}
