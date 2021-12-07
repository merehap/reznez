use minifb::{Key, Window, WindowOptions, Scale};

use crate::nes::Nes;
use crate::ppu::screen::Screen;

pub fn gui(mut nes: Nes) {
    let mut window_options = WindowOptions::default();
    window_options.scale = Scale::X4;
    let mut window = Window::new(
        "Test - ESC to exit",
        Screen::WIDTH,
        Screen::HEIGHT,
        window_options,
    ).unwrap_or_else(|e| {
        panic!("{}", e);
    });

    while window.is_open() && !window.is_key_down(Key::Escape) {
        nes.step();

        if nes.ppu().clock().scanline() == 0 && nes.ppu().clock().cycle() == 0 {
            let mut pixels = Vec::new();
            for row in 0..Screen::HEIGHT {
                for column in 0..Screen::WIDTH {
                    let pixel = nes.ppu().screen().pixel(column as u8, row as u8);
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
