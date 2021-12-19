use minifb::{Key, Window, WindowOptions, Scale};
use stopwatch::Stopwatch;

use crate::nes::Nes;
use crate::ppu::palette::palette::Palette;
use crate::ppu::palette::rgb::Rgb;
use crate::ppu::screen::Screen;

const DEBUG_SCREEN_HEIGHT: usize = 20;

pub fn gui(mut nes: Nes) {
    let window_options = WindowOptions {scale: Scale::X4, ..Default::default()};
    let mut window = Window::new(
        "Test - ESC to exit",
        Screen::WIDTH,
        Screen::HEIGHT + DEBUG_SCREEN_HEIGHT,
        window_options,
    ).unwrap_or_else(|e| {
        panic!("{}", e);
    });

    let mut screen = Screen::new();

    let totalwatch = Stopwatch::start_new();
    let mut framewatch = Stopwatch::start_new();

    let mut palette_screen =
        DebugScreen::<{Screen::WIDTH}, DEBUG_SCREEN_HEIGHT>::new(Rgb::WHITE);

    while window.is_open() && !window.is_key_down(Key::Escape) {
        while window.is_key_down(Key::Enter) {}

        nes.step(&mut screen);

        if nes.ppu().clock().scanline() == 1 && nes.ppu().clock().cycle() == 1 {
            let frame = nes.ppu().clock().frame();
            println!(
                "Frame: {}, Rate: {}, Average: {}",
                nes.ppu().clock().frame(),
                1_000_000_000.0 / framewatch.elapsed().as_nanos() as f64,
                1000.0 / totalwatch.elapsed_ms() as f64 * frame as f64,
                );

            framewatch = Stopwatch::start_new();

            let mut pixels = Vec::new();
            for row in 0..Screen::HEIGHT {
                for column in 0..Screen::WIDTH {
                    let pixel = screen.pixel(column as u8, row as u8);
                    let value =
                        ((pixel.red()   as u32) << 16) +
                        ((pixel.green() as u32) <<  8) +
                         (pixel.blue()  as u32);
                    pixels.push(value);
                }
            }

            let palette_table = nes.ppu().palette_table();

            let mut add_palettes_to_screen =
                |palettes: [Palette; 4], vertical_offset: usize| -> () {
                    for (index, palette) in palettes.iter().enumerate() {
                        for (color_column, rgb) in palette.rgbs().iter().enumerate() {
                            for pixel_column in 0..8 {
                                let pixel_column = 40 * index as usize + 10 * color_column + pixel_column;
                                for pixel_row in 0..8 {
                                    let pixel_row = pixel_row + vertical_offset;
                                    palette_screen.set_pixel(pixel_column, pixel_row, *rgb);
                                }
                            }
                        }
                    }
                };

            add_palettes_to_screen(palette_table.background_palettes(), 0);
            add_palettes_to_screen(palette_table.sprite_palettes(), 10);

            for row in 0..palette_screen.height() {
                for column in 0..palette_screen.width() {
                    let pixel = palette_screen.pixel(column, row);
                    let value =
                        ((pixel.red()   as u32) << 16) +
                        ((pixel.green() as u32) <<  8) +
                         (pixel.blue()  as u32);
                    pixels.push(value);
                }
            }

            window
                .update_with_buffer(&pixels, Screen::WIDTH, Screen::HEIGHT + DEBUG_SCREEN_HEIGHT)
                .unwrap();
        }
    }
}

pub struct DebugScreen<const WIDTH: usize, const HEIGHT: usize> {
    buffer: [[Rgb; WIDTH]; HEIGHT],
}

impl <const WIDTH: usize, const HEIGHT: usize> DebugScreen<WIDTH, HEIGHT> {
    pub fn new(default_rgb: Rgb) -> DebugScreen<WIDTH, HEIGHT> {
        DebugScreen {
            buffer: [[default_rgb; WIDTH]; HEIGHT],
        }
    }

    pub fn width(&self) -> usize {
        WIDTH
    }

    pub fn height(&self) -> usize {
        HEIGHT
    }

    pub fn pixel(&self, column: usize, row: usize) -> Rgb {
        self.buffer[row][column]
    }

    pub fn set_pixel(&mut self, column: usize, row: usize, rgb: Rgb) {
        self.buffer[row][column] = rgb;
    }
}
