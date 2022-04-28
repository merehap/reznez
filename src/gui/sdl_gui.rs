use std::collections::BTreeMap;
use std::collections::HashMap;

use lazy_static::lazy_static;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use sdl2::render::{Canvas, Texture};
use sdl2::video::Window;
use sdl2::EventPump;

use crate::config::Config;
use crate::controller::joypad::{Button, ButtonStatus};
use crate::gui::gui::{execute_frame, Events, Gui};
use crate::nes::Nes;
use crate::ppu::pixel_index::{PixelColumn, PixelIndex, PixelRow};
use crate::ppu::render::frame::Frame;

lazy_static! {
    #[rustfmt::skip]
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

    #[rustfmt::skip]
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
    pixels: [u8; 3 * PixelIndex::PIXEL_COUNT],
}

impl SdlGui {
    pub fn new() -> SdlGui {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();
        let window = video_subsystem
            .window(
                "REZNEZ",
                (PixelColumn::COLUMN_COUNT * 3) as u32,
                (PixelRow::ROW_COUNT * 3) as u32,
            )
            .position_centered()
            .build()
            .unwrap();

        let mut canvas = window.into_canvas().present_vsync().build().unwrap();
        canvas.set_scale(3.0, 3.0).unwrap();

        let texture_creator = canvas.texture_creator();
        let texture = texture_creator
            .create_texture_target(
                PixelFormatEnum::RGB24,
                PixelColumn::COLUMN_COUNT as u32,
                PixelRow::ROW_COUNT as u32,
            )
            .unwrap();

        SdlGui {
            event_pump: sdl_context.event_pump().unwrap(),

            canvas,
            texture,
            pixels: [0; 3 * PixelIndex::PIXEL_COUNT],
        }
    }

    #[inline]
    fn events(&mut self) -> Events {
        let mut should_quit = false;
        let mut joypad1_button_statuses = BTreeMap::new();
        let mut joypad2_button_statuses = BTreeMap::new();

        for event in self.event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    should_quit = true
                }
                Event::KeyDown { keycode: Some(code), .. } => {
                    if let Some(&button) = JOY_1_BUTTON_MAPPINGS.get(&code) {
                        joypad1_button_statuses.insert(button, ButtonStatus::Pressed);
                    }

                    if let Some(&button) = JOY_2_BUTTON_MAPPINGS.get(&code) {
                        joypad2_button_statuses.insert(button, ButtonStatus::Pressed);
                    }
                }
                Event::KeyUp { keycode: Some(code), .. } => {
                    if let Some(&button) = JOY_1_BUTTON_MAPPINGS.get(&code) {
                        joypad1_button_statuses.insert(button, ButtonStatus::Unpressed);
                    }

                    if let Some(&button) = JOY_2_BUTTON_MAPPINGS.get(&code) {
                        joypad2_button_statuses.insert(button, ButtonStatus::Unpressed);
                    }
                }
                _ => { /* Do nothing. */ }
            }
        }

        Events {
            should_quit,
            joypad1_button_statuses,
            joypad2_button_statuses,
        }
    }
}

impl Gui for SdlGui {
    fn run(&mut self, mut nes: Nes, config: Config) {
        loop {
            let events = self.events();
            let display_frame = |frame: &Frame, mask, _frame_index| {
                self.pixels = frame.write_all_pixel_data(mask, self.pixels);
                self.texture.update(None, &self.pixels, 256 * 3).unwrap();
                self.canvas.copy(&self.texture, None, None).unwrap();
                self.canvas.present();
            };
            execute_frame(&mut nes, &config, events, display_frame);
        }
    }
}
