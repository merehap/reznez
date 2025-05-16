use std::ops::{Index, IndexMut};

use enum_iterator::IntoEnumIterator;

use crate::ppu::palette::rgb::Rgb;
use crate::ppu::palette::rgbt::Rgbt;
use crate::ppu::pattern_table::Tile;
use crate::ppu::pixel_index::{
    ColumnInTile, PixelColumn, PixelIndex, PixelRow, RowInTile,
};
use crate::ppu::register::registers::mask::Mask;
use crate::ppu::render::ppm::Ppm;
use crate::ppu::sprite::sprite_attributes::Priority;

#[derive(Clone)]
pub struct Frame {
    buffer: FrameBuffer<Rgbt>,
    sprite_buffer: FrameBuffer<(Rgbt, Priority, bool)>,
    universal_background_rgb: Rgb,

    show_overscan: bool,
}

impl Frame {
    pub fn new() -> Frame {
        Frame {
            buffer: FrameBuffer::filled(Rgbt::Transparent),
            sprite_buffer: FrameBuffer::filled((
                Rgbt::Transparent,
                Priority::Behind,
                false,
            )),
            universal_background_rgb: Rgb::BLACK,

            show_overscan: false,
        }
    }

    // Only used for debug windows.
    pub fn to_background_only(&self) -> Frame {
        let mut frame = self.clone();
        frame.sprite_buffer =
            FrameBuffer::filled((Rgbt::Transparent, Priority::Behind, false));
        frame.universal_background_rgb = Rgb::BLACK;
        frame
    }

    pub fn show_overscan_mut(&mut self) -> &mut bool {
        &mut self.show_overscan
    }

    pub fn pixel(&self, mask: Mask, column: PixelColumn, row: PixelRow) -> (Rgb, Sprite0Hit) {
        use Rgbt::{Opaque, Transparent};
        let mut background_pixel = self.buffer[(column, row)];
        if !mask.left_background_columns_enabled() && column.is_in_left_margin() {
            background_pixel = Transparent;
        }

        let (mut sprite_pixel, sprite_priority, is_sprite_0) =
            self.sprite_buffer[(column, row)];
        if !mask.left_sprite_columns_enabled() && column.is_in_left_margin() {
            sprite_pixel = Transparent;
        }

        use Sprite0Hit::{Hit, Miss};
        let sprite_0_hit = if is_sprite_0 { Hit } else { Miss };

        // https://wiki.nesdev.org/w/index.php?title=PPU_OAM#Sprite_zero_hits
        use Priority::{Behind, InFront};
        let (mut rgb, sprite_0_hit) = match (background_pixel, sprite_pixel, sprite_priority, column) {
            (Transparent, Transparent, _, _) => (self.universal_background_rgb, Miss),
            (Transparent, Opaque(rgb), _, _) => (rgb, Miss),
            (Opaque(rgb), Transparent, _, _) => (rgb, Miss),
            (Opaque(_), Opaque(rgb), InFront, PixelColumn::MAX) => (rgb, Miss),
            (Opaque(rgb), Opaque(_), Behind, PixelColumn::MAX) => (rgb, Miss),
            (Opaque(_), Opaque(rgb), InFront, _) => (rgb, sprite_0_hit),
            (Opaque(rgb), Opaque(_), Behind, _) => (rgb, sprite_0_hit),
        };

        if mask.greyscale_enabled() {
            rgb = rgb.to_greyscale();
        }

        if !self.show_overscan && (column.is_in_overscan_region() || row.is_in_overscan_region()) {
            rgb = Rgb::BLACK;
        }

        (rgb, sprite_0_hit)
    }

    pub fn set_universal_background_rgb(&mut self, rgb: Rgb) {
        self.universal_background_rgb = rgb;
    }

    pub fn clear(&mut self) {
        self.buffer = FrameBuffer::filled(Rgbt::Transparent);
        self.sprite_buffer =
            FrameBuffer::filled((Rgbt::Transparent, Priority::Behind, false));
        self.universal_background_rgb = Rgb::BLACK;
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
        rgbt: Rgbt,
        priority: Priority,
        is_sprite_0: bool,
    ) {
        self.sprite_buffer[(column, row)] = (rgbt, priority, is_sprite_0);
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
            data[index] = pixel.red();
            data[index + 1] = pixel.green();
            data[index + 2] = pixel.blue();
        }

        data
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
            buffer[index] = pixel.red();
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

#[derive(Clone)]
struct FrameBuffer<T>(Box<[[T; PixelColumn::COLUMN_COUNT]; PixelRow::ROW_COUNT]>);

impl<T: Copy> FrameBuffer<T> {
    fn filled(value: T) -> FrameBuffer<T> {
        FrameBuffer(Box::new(
            [[value; PixelColumn::COLUMN_COUNT]; PixelRow::ROW_COUNT],
        ))
    }
}

impl<T> Index<(PixelColumn, PixelRow)> for FrameBuffer<T> {
    type Output = T;

    fn index(&self, (column, row): (PixelColumn, PixelRow)) -> &T {
        &self.0[row.to_usize()][column.to_usize()]
    }
}

impl<T> IndexMut<(PixelColumn, PixelRow)> for FrameBuffer<T> {
    fn index_mut(&mut self, (column, row): (PixelColumn, PixelRow)) -> &mut T {
        &mut self.0[row.to_usize()][column.to_usize()]
    }
}

pub struct DebugBuffer<const WIDTH: usize, const HEIGHT: usize> {
    buffer: Box<[[Rgbt; WIDTH]; HEIGHT]>,
    background_rgb: Rgb,
}

impl<const WIDTH: usize, const HEIGHT: usize> DebugBuffer<WIDTH, HEIGHT> {
    pub fn new(background_rgb: Rgb) -> DebugBuffer<WIDTH, HEIGHT> {
        DebugBuffer {
            buffer: Box::new([[Rgbt::Transparent; WIDTH]; HEIGHT]),
            background_rgb,
        }
    }

    pub fn place_frame(&mut self, left_column: usize, top_row: usize, frame: &Frame) {
        let mask = Mask::full_screen_enabled();
        for pixel_index in PixelIndex::iter() {
            let (column, row) = pixel_index.to_column_row();
            let (pixel, _) = frame.pixel(mask, column, row);
            self.write(
                left_column + column.to_usize(),
                top_row + row.to_usize(),
                pixel,
            );
        }
    }

    pub fn place_tile(&mut self, left_column: usize, top_row: usize, tile: &Tile) {
        for row_in_tile in RowInTile::into_enum_iter() {
            for column_in_tile in ColumnInTile::into_enum_iter() {
                let column_in_tile = column_in_tile as usize;
                let row_in_tile = row_in_tile as usize;
                self.write_rgbt(
                    left_column + column_in_tile,
                    top_row + row_in_tile,
                    tile.0[row_in_tile][column_in_tile],
                );
            }
        }
    }

    pub fn place_wrapping_horizontal_line(
        &mut self,
        row: usize,
        left_column: usize,
        right_column: usize,
        rgb: Rgb,
    ) {
        let row = row.rem_euclid(HEIGHT);
        let left_column = left_column.rem_euclid(WIDTH);
        let right_column = right_column.rem_euclid(WIDTH);
        if left_column < right_column {
            for column in left_column..=right_column {
                self.write(column, row, rgb);
            }
        } else {
            for column in left_column..WIDTH {
                self.write(column, row, rgb);
            }

            for column in 0..=right_column {
                self.write(column, row, rgb);
            }
        }
    }

    pub fn place_wrapping_vertical_line(
        &mut self,
        column: usize,
        top_row: usize,
        bottom_row: usize,
        rgb: Rgb,
    ) {
        let column = column.rem_euclid(WIDTH);
        let top_row = top_row.rem_euclid(HEIGHT);
        let bottom_row = bottom_row.rem_euclid(HEIGHT);
        if top_row < bottom_row {
            for row in top_row..=bottom_row {
                self.write(column, row, rgb);
            }
        } else {
            for row in top_row..HEIGHT {
                self.write(column, row, rgb);
            }

            for row in 0..=bottom_row {
                self.write(column, row, rgb);
            }
        }
    }

    pub fn copy_to_rgba_buffer(&self, buffer: &mut [u8]) {
        for row in 0..HEIGHT {
            for column in 0..WIDTH {
                let index = 4 * (WIDTH * row + column);
                let pixel = self.read(column, row);
                buffer[index] = pixel.red();
                buffer[index + 1] = pixel.green();
                buffer[index + 2] = pixel.blue();
                // No transparency.
                buffer[index + 3] = 0xFF;
            }
        }
    }

    fn read(&self, column: usize, row: usize) -> Rgb {
        self.buffer[row][column]
            .to_rgb()
            .unwrap_or(self.background_rgb)
    }

    fn write(&mut self, column: usize, row: usize, rgb: Rgb) {
        self.buffer[row][column] = Rgbt::Opaque(rgb);
    }

    fn write_rgbt(&mut self, column: usize, row: usize, rgbt: Rgbt) {
        self.buffer[row][column] = rgbt;
    }
}
