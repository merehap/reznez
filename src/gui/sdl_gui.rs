use std::collections::BTreeMap;
use std::collections::HashMap;

use lazy_static::lazy_static;
use sdl2::EventPump;
use sdl2::event::Event;
//use sdl2::gfx::framerate::FPSManager;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use sdl2::render::{Canvas, Texture};
use sdl2::video::Window;

use crate::controller::joypad::{Button, ButtonStatus};
use crate::gui::gui::{Gui, Events};
use crate::ppu::render::frame::Frame;

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

pub struct SdlGui {
    event_pump: EventPump,

    canvas: Canvas<Window>,
    texture: Texture,
    frame: Frame,
    pixels: [u8; 3 * Frame::WIDTH * Frame::HEIGHT],
}

impl Gui for SdlGui {
    fn initialize() -> SdlGui {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();
        let window = video_subsystem
            .window("REZNEZ", (256.0 * 3.0) as u32, (240.0 * 3.0) as u32)
            .position_centered()
            .build()
            .unwrap();

        let mut canvas = window.into_canvas().present_vsync().build().unwrap();
        canvas.set_scale(3.0, 3.0).unwrap();

        let texture_creator = canvas.texture_creator();
        let texture = texture_creator
            .create_texture_target(PixelFormatEnum::RGB24, 256, 240)
            .unwrap();

        /*
        let palette_screen =
            DebugScreen::<{Screen::WIDTH}, DEBUG_SCREEN_HEIGHT>::new(Rgb::WHITE);
            */
        // TODO: Figure out how to enable this. Currently get a linking error for feature gfx.
        //FPSManager::new().set_framerate(100_000);
        SdlGui {
            event_pump: sdl_context.event_pump().unwrap(),

            canvas,
            texture,
            frame: Frame::new(),
            pixels: [0; 3 * Frame::WIDTH * Frame::HEIGHT],
        }
    }

    #[inline]
    fn events(&mut self) -> Events {
        let mut should_quit = false;
        let mut joypad_1_button_statuses = BTreeMap::new();
        let mut joypad_2_button_statuses = BTreeMap::new();

        for event in self.event_pump.poll_iter() {
            match event {
                Event::Quit { .. } |
                Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => should_quit = true,
                Event::KeyDown {keycode: Some(code), ..} => {
                    if let Some(&button) = JOY_1_BUTTON_MAPPINGS.get(&code) {
                        joypad_1_button_statuses.insert(button, ButtonStatus::Pressed);
                    }

                    if let Some(&button) = JOY_2_BUTTON_MAPPINGS.get(&code) {
                        joypad_2_button_statuses.insert(button, ButtonStatus::Pressed);
                    }
                },
                Event::KeyUp {keycode: Some(code), ..} => {
                    if let Some(&button) = JOY_1_BUTTON_MAPPINGS.get(&code) {
                        joypad_1_button_statuses.insert(button, ButtonStatus::Unpressed);
                    }

                    if let Some(&button) = JOY_2_BUTTON_MAPPINGS.get(&code) {
                        joypad_2_button_statuses.insert(button, ButtonStatus::Unpressed);
                    }
                },
                _ => { /* Do nothing. */ }
            }
        }

        Events {
            should_quit,
            joypad_1_button_statuses,
            joypad_2_button_statuses,
        }
    }

    fn frame_mut(&mut self) -> &mut Frame {
        &mut self.frame
    }

    fn display_frame(&mut self, _frame_index: u64) {
        self.pixels = self.frame.write_all_pixel_data(self.pixels);

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

        self.texture.update(None, &self.pixels, 256 * 3).unwrap();

        self.canvas.copy(&self.texture, None, None).unwrap();

        self.canvas.present();
    }
}
