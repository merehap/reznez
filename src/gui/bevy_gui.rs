use std::collections::{BTreeMap, HashMap};

use bevy::input::keyboard::KeyCode;
use bevy::prelude::*;
use bevy_pixels::prelude::*;
use lazy_static::lazy_static;

use crate::config::Config;
use crate::controller::joypad::{Button, ButtonStatus};
use crate::gui::gui::{execute_frame, Events, Gui};
use crate::nes::Nes;
use crate::ppu::render::frame::Frame;

lazy_static! {
    #[rustfmt::skip]
    static ref JOY_1_BUTTON_MAPPINGS: HashMap<KeyCode, Button> = {
        let mut mappings = HashMap::new();
        mappings.insert(KeyCode::Space,  Button::A);
        mappings.insert(KeyCode::F,      Button::B);
        mappings.insert(KeyCode::RShift, Button::Select);
        mappings.insert(KeyCode::Return, Button::Start);
        mappings.insert(KeyCode::Up,     Button::Up);
        mappings.insert(KeyCode::Down,   Button::Down);
        mappings.insert(KeyCode::Left,   Button::Left);
        mappings.insert(KeyCode::Right,  Button::Right);
        mappings
    };

    #[rustfmt::skip]
    static ref JOY_2_BUTTON_MAPPINGS: HashMap<KeyCode, Button> = {
        let mut mappings = HashMap::new();
        mappings.insert(KeyCode::Numpad0,        Button::A);
        mappings.insert(KeyCode::NumpadEnter,    Button::B);
        mappings.insert(KeyCode::NumpadSubtract, Button::Select);
        mappings.insert(KeyCode::NumpadAdd,      Button::Start);
        mappings.insert(KeyCode::Numpad8,        Button::Up);
        mappings.insert(KeyCode::Numpad5,        Button::Down);
        mappings.insert(KeyCode::Numpad4,        Button::Left);
        mappings.insert(KeyCode::Numpad6,        Button::Right);
        mappings
    };
}

pub struct BevyGui {
    app: App,
}

impl BevyGui {
    pub fn new() -> BevyGui {
        let mut gui = BevyGui { app: App::new() };
        gui.app
            .insert_resource(PixelsOptions { width: 256, height: 240 })
            // Default plugins, minus logging.
            .add_plugin(bevy::core::CorePlugin)
            .add_plugin(bevy::diagnostic::DiagnosticsPlugin)
            .add_plugin(bevy::input::InputPlugin)
            .add_plugin(bevy::window::WindowPlugin {
                add_primary_window: true,
                exit_on_close: true,
            })
            .add_plugin(bevy::asset::AssetPlugin)
            .add_plugin(bevy::scene::ScenePlugin)
            .add_plugin(bevy::winit::WinitPlugin)
            // REZNEZ-specific.
            .add_plugin(PixelsPlugin)
            .add_system(main_system);

        gui
    }
}

impl Gui for BevyGui {
    fn run(&mut self, nes: Nes, config: Config) {
        self.app
            .insert_non_send_resource(nes)
            .insert_non_send_resource(config)
            .run()
    }
}

fn main_system(
    mut nes: NonSendMut<Nes>,
    config: NonSend<Config>,
    keyboard_input: Res<Input<KeyCode>>,
    mut pixels: ResMut<PixelsResource>,
) {
    let events = events(keyboard_input);
    let display_frame = |frame: &Frame, mask, _frame_index| {
        frame.copy_to_rgba_buffer(mask, pixels.pixels.get_frame().try_into().unwrap());
    };
    execute_frame(&mut nes, &config, events, display_frame);
}

fn events(keyboard_input: Res<Input<KeyCode>>) -> Events {
    let mut joypad1_button_statuses = BTreeMap::new();
    let mut joypad2_button_statuses = BTreeMap::new();

    for key_code in keyboard_input.get_just_pressed() {
        if let Some(&button) = JOY_1_BUTTON_MAPPINGS.get(key_code) {
            joypad1_button_statuses.insert(button, ButtonStatus::Pressed);
        }

        if let Some(&button) = JOY_2_BUTTON_MAPPINGS.get(key_code) {
            joypad2_button_statuses.insert(button, ButtonStatus::Pressed);
        }
    }

    for key_code in keyboard_input.get_just_released() {
        if let Some(&button) = JOY_1_BUTTON_MAPPINGS.get(key_code) {
            joypad1_button_statuses.insert(button, ButtonStatus::Unpressed);
        }

        if let Some(&button) = JOY_2_BUTTON_MAPPINGS.get(key_code) {
            joypad2_button_statuses.insert(button, ButtonStatus::Unpressed);
        }
    }

    let should_quit = keyboard_input.pressed(KeyCode::Escape);
    Events {
        should_quit,
        joypad1_button_statuses,
        joypad2_button_statuses,
    }
}
