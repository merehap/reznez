use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use fltk::{app, enums::{Event, Key}, prelude::*, window::Window};
use lazy_static::lazy_static;
use log::error;
use pixels::{Error, Pixels, SurfaceTexture};

use stopwatch::Stopwatch;

use crate::controller::joypad::Button;
use crate::nes::Nes;
use crate::ppu::palette::palette::Palette;
use crate::ppu::palette::rgb::Rgb;
use crate::ppu::screen::Screen;

const DEBUG_SCREEN_HEIGHT: usize = 20;

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

pub fn gui(nes: Nes) {
    #[cfg(debug_assertions)]
    env_logger::init();

    let app = app::App::default();
    let mut win = Window::default()
        .with_size(4 * Screen::WIDTH as i32, 4 * Screen::HEIGHT as i32)
        .with_label("Hello Pixels");
    win.make_resizable(false);
    win.end();
    win.show();

    let mut pixels = {
        let pixel_width = win.pixel_w() as u32;
        let pixel_height = win.pixel_h() as u32;
        let surface_texture = SurfaceTexture::new(pixel_width, pixel_height, &win);
        Pixels::new(Screen::WIDTH as u32, Screen::HEIGHT as u32, surface_texture).unwrap()
    };

    let nes = Arc::new(Mutex::new(nes));
    let mut world = World::new(nes.clone());

    win.handle(move |_, ev| match ev {
        Event::KeyDown => {
            if let Some(&button) = BUTTON_MAPPINGS.get(&app::event_key()) {
                println!("Button {:?} pressed.", button);
                nes.lock().unwrap().joypad_1.press_button(button);
                return true;
            }

            false
        },
        Event::KeyUp => {
            if let Some(&button) = BUTTON_MAPPINGS.get(&app::event_key()) {
                println!("Button {:?} released.", button);
                nes.lock().unwrap().joypad_1.press_button(button);
                return true;
            }

            false
        },
        _ => false,
    });

    while app.wait() {
        // Update internal state
        while !world.update() {}

        // Draw the current frame
        world.draw(pixels.get_frame());
        if pixels
            .render()
            .map_err(|e| error!("pixels.render() failed: {}", e))
            .is_err()
        {
            app.quit();
        }

        app::flush();
        app::awake();
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
        let frame = nes.ppu().clock().frame();
        println!(
            "Frame: {}, Rate: {}, Average: {}",
            frame,
            1_000_000_000.0 / self.framewatch.elapsed().as_nanos() as f64,
            1000.0 / self.totalwatch.elapsed_ms() as f64 * frame as f64,
            );

        self.framewatch = Stopwatch::start_new();

        let mut rgb_count = 0;
        let mut set_next_pixel = |rgb: Rgb| {
            pixels[4 * rgb_count + 0] = rgb.red();
            pixels[4 * rgb_count + 1] = rgb.green();
            pixels[4 * rgb_count + 2] = rgb.blue();
            pixels[4 * rgb_count + 3] = 0;
            rgb_count += 1;
        };

        for row in 0..Screen::HEIGHT {
            for column in 0..Screen::WIDTH {
                let pixel = self.screen.pixel(column as u8, row as u8);
                set_next_pixel(pixel);
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
                                self.palette_screen.set_pixel(pixel_column, pixel_row, *rgb);
                            }
                        }
                    }
                }
            };

        add_palettes_to_screen(palette_table.background_palettes(), 0);
        add_palettes_to_screen(palette_table.sprite_palettes(), 10);

        /*
        for row in 0..self.palette_screen.height() {
            for column in 0..self.palette_screen.width() {
                let pixel = self.palette_screen.pixel(column, row);
                set_next_pixel(pixel);
            }
        }
        */
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
