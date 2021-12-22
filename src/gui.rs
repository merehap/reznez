use std::collections::HashMap;

use lazy_static::lazy_static;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;

use stopwatch::Stopwatch;

use crate::controller::joypad::Button;
use crate::nes::Nes;
use crate::ppu::palette::rgb::Rgb;
use crate::ppu::screen::Screen;

const DEBUG_SCREEN_HEIGHT: usize = 20;

lazy_static! {
    static ref JOY_1_BUTTON_MAPPINGS: HashMap<Keycode, Button> = {
        let mut mappings = HashMap::new();
        mappings.insert(Keycode::Space,  Button::A);
        mappings.insert(Keycode::F,      Button::B);
        mappings.insert(Keycode::RShift, Button::Select);
        mappings.insert(Keycode::Return, Button::Start);
        mappings.insert(Keycode::Up,     Button::Up);
        mappings.insert(Keycode::Down,   Button::Down);
        mappings.insert(Keycode::Left,   Button::Left);
        mappings.insert(Keycode::Right,  Button::Right);
        mappings
    };

    static ref JOY_2_BUTTON_MAPPINGS: HashMap<Keycode, Button> = {
        let mut mappings = HashMap::new();
        mappings.insert(Keycode::Kp0,     Button::A);
        mappings.insert(Keycode::KpEnter, Button::B);
        mappings.insert(Keycode::KpMinus, Button::Select);
        mappings.insert(Keycode::KpPlus,  Button::Start);
        mappings.insert(Keycode::Kp8,     Button::Up);
        mappings.insert(Keycode::Kp5,     Button::Down);
        mappings.insert(Keycode::Kp4,     Button::Left);
        mappings.insert(Keycode::Kp6,     Button::Right);
        mappings
    };
}

pub fn gui(mut nes: Nes) {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window("REZNEZ", (256.0 * 3.0) as u32, (240.0 * 3.0) as u32)
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
    let totalwatch = Stopwatch::start_new();
    let mut framewatch = Stopwatch::start_new();
    /*
    let palette_screen =
        DebugScreen::<{Screen::WIDTH}, DEBUG_SCREEN_HEIGHT>::new(Rgb::WHITE);
        */

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

            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit { .. } |
                    Event::KeyDown {
                        keycode: Some(Keycode::Escape),
                        ..
                    } => std::process::exit(0),
                    Event::KeyDown {keycode: Some(code), ..} => {
                        if let Some(&button) = JOY_1_BUTTON_MAPPINGS.get(&code) {
                            nes.joypad_1.press_button(button);
                        } else if let Some(&button) = JOY_2_BUTTON_MAPPINGS.get(&code) {
                            nes.joypad_2.press_button(button);
                        }
                    },
                    Event::KeyUp {keycode: Some(code), ..} => {
                        if let Some(&button) = JOY_1_BUTTON_MAPPINGS.get(&code) {
                            nes.joypad_1.release_button(button);
                        } else if let Some(&button) = JOY_2_BUTTON_MAPPINGS.get(&code) {
                            nes.joypad_2.release_button(button);
                        }
                    },
                    _ => { /* do nothing */ }
                }
            }
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
