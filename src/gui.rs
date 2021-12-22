use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use lazy_static::lazy_static;
use log::error;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::pixels::PixelFormatEnum;
use sdl2::EventPump;

use stopwatch::Stopwatch;

use crate::controller::joypad::Button;
use crate::nes::Nes;
use crate::ppu::palette::palette::Palette;
use crate::ppu::palette::rgb::Rgb;
use crate::ppu::screen::Screen;

const DEBUG_SCREEN_HEIGHT: usize = 20;

/*
lazy_static! {
    static ref BUTTON_MAPPINGS: HashMap<Key, Button> = {
        let mut mappings = HashMap::new();
        mappings.insert(Key::from_char('a'),      Button::A);
        mappings.insert(Key::from_char('s'),      Button::B);
        mappings.insert(Key::from_char('z'),  Button::Select);
        mappings.insert(Key::Enter, Button::Start);
        mappings.insert(Key::Up,     Button::Up);
        mappings.insert(Key::Down,   Button::Down);
        mappings.insert(Key::Left,   Button::Left);
        mappings.insert(Key::Right,  Button::Right);
        mappings
    };
}
*/

pub fn gui(mut nes: Nes) {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window("Tile viewer", (256.0 * 3.0) as u32, (240.0 * 3.0) as u32)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().present_vsync().build().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();
    canvas.set_scale(3.0, 3.0).unwrap();

    let creator = canvas.texture_creator();
    let mut texture = creator
        .create_texture_target(PixelFormatEnum::RGB24, 256, 240)
        .unwrap();

    let mut screen = Screen::new();
    let mut totalwatch = Stopwatch::start_new();
    let mut framewatch = Stopwatch::start_new();
    let palette_screen =
        DebugScreen::<{Screen::WIDTH}, DEBUG_SCREEN_HEIGHT>::new(Rgb::WHITE);

    let mut pixels = [0; 4 * Screen::WIDTH * Screen::HEIGHT];

    loop {
        nes.step(&mut screen);
        let should_redraw = nes.ppu().clock().is_first_cycle_of_frame();
        if should_redraw {
            let frame = nes.ppu().clock().frame();
            println!(
                "Frame: {}, Rate: {}, Average: {}",
                frame,
                1_000_000_000.0 / framewatch.elapsed().as_nanos() as f64,
                1000.0 / totalwatch.elapsed_ms() as f64 * frame as f64,
                );

            framewatch = Stopwatch::start_new();

            let mut rgb_count = 0;
            let mut set_next_pixel = |rgb: Rgb| {
                pixels[3 * rgb_count + 0] = rgb.red();
                pixels[3 * rgb_count + 1] = rgb.green();
                pixels[3 * rgb_count + 2] = rgb.blue();
                rgb_count += 1;
            };

            for row in 0..Screen::HEIGHT {
                for column in 0..Screen::WIDTH {
                    let pixel = screen.pixel(column as u8, row as u8);
                    set_next_pixel(pixel);
                }
            }

            /*
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
                */

            //add_palettes_to_screen(palette_table.background_palettes(), 0);
            //add_palettes_to_screen(palette_table.sprite_palettes(), 10);

            /*
            for row in 0..palette_screen.height() {
                for column in 0..palette_screen.width() {
                    let pixel = palette_screen.pixel(column, row);
                    set_next_pixel(pixel);
                }
            }
            */

            texture.update(None, &pixels, 256 * 3).unwrap();

            canvas.copy(&texture, None, None).unwrap();

            canvas.present();
        }

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } | Event::KeyDown {
                  keycode: Some(Keycode::Escape),
                  ..
                } => std::process::exit(0),
                _ => { /* do nothing */ }
            }
        }
    }
}

struct World {
    nes: Arc<Mutex<Nes>>,

    screen: Screen,
    totalwatch: Stopwatch,
    framewatch: Stopwatch,
    palette_screen: DebugScreen::<{Screen::WIDTH}, DEBUG_SCREEN_HEIGHT>,

}

impl World {
    fn new(nes: Arc<Mutex<Nes>>) -> Self {
        Self {
            nes,
            screen: Screen::new(),
            totalwatch: Stopwatch::start_new(),
            framewatch: Stopwatch::start_new(),
            palette_screen:
                DebugScreen::<{Screen::WIDTH}, DEBUG_SCREEN_HEIGHT>::new(Rgb::WHITE),
        }
    }

    fn update(&mut self) -> bool {
        self.nes.lock().unwrap().step(&mut self.screen);

        let should_redraw = self.nes.lock().unwrap().ppu().clock().is_first_cycle_of_frame();
        should_redraw
    }

    fn draw(&mut self, pixels: &mut [u8]) {
        let nes = self.nes.lock().unwrap();
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
