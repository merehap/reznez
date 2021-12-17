use minifb::{Key, Window, WindowOptions, Scale};
use stopwatch::Stopwatch;

use crate::nes::Nes;
use crate::ppu::screen::Screen;

pub fn gui(mut nes: Nes) {
    let window_options = WindowOptions {scale: Scale::X4, ..Default::default()};
    let mut window = Window::new(
        "Test - ESC to exit",
        Screen::WIDTH,
        Screen::HEIGHT,
        window_options,
    ).unwrap_or_else(|e| {
        panic!("{}", e);
    });

    let mut screen = Screen::new();

    let totalwatch = Stopwatch::start_new();
    let mut framewatch = Stopwatch::start_new();

    while window.is_open() && !window.is_key_down(Key::Escape) {
        nes.step(&mut screen);

        if nes.ppu().clock().scanline() == 1 && nes.ppu().clock().cycle() == 1 {
            let frame = nes.ppu().clock().frame();
            println!(
                "Frame: {}, Rate: {}, Average: {}",
                nes.ppu().clock().frame(),
                1000.0 / framewatch.elapsed_ms() as f64,
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

            window
                .update_with_buffer(&pixels, Screen::WIDTH, Screen::HEIGHT)
                .unwrap();
        }
    }
}
