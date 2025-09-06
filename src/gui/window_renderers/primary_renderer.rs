use std::collections::{BTreeMap, HashMap};
use std::sync::LazyLock;

pub use egui::Context;
use gilrs::GamepadId;
use pixels::Pixels;
pub use winit::dpi::{PhysicalPosition, Position};
use winit::keyboard::KeyCode;
use winit_input_helper::WinitInputHelper;

use crate::controller::joypad::{Button, ButtonStatus};
use crate::gui::gui::{execute_frame, Events};
pub use crate::gui::window_renderer::{FlowControl, WindowRenderer};
use crate::gui::window_renderers::cartridge_metadata_renderer::CartridgeMetadataRenderer;
use crate::gui::window_renderers::cartridge_query_renderer::CartridgeQueryRenderer;
use crate::gui::window_renderers::display_settings_renderer::DisplaySettingsRenderer;
use crate::gui::window_renderers::layers_renderer::LayersRenderer;
pub use crate::gui::window_renderers::load_rom_renderer::LoadRomRenderer;
use crate::gui::window_renderers::memory_viewer_renderer::MemoryViewerRenderer;
use crate::gui::window_renderers::name_table_renderer::NameTableRenderer;
use crate::gui::window_renderers::pattern_source_renderer::PatternSourceRenderer;
use crate::gui::window_renderers::pattern_table_renderer::PatternTableRenderer;
use crate::gui::window_renderers::sprites_renderer::SpritesRenderer;
use crate::gui::window_renderers::status_renderer::StatusRenderer;
pub use crate::gui::world::World;
use crate::ppu::pixel_index::{PixelColumn, PixelRow};
use crate::ppu::render::frame::Frame;

#[rustfmt::skip]
static JOY_1_KEYBOARD_MAPPINGS: LazyLock<HashMap<KeyCode, Button>> = LazyLock::new(|| {
    let mut mappings = HashMap::new();
    mappings.insert(KeyCode::KeyJ,   Button::B);
    mappings.insert(KeyCode::KeyK,   Button::A);
    mappings.insert(KeyCode::KeyU,   Button::Select);
    mappings.insert(KeyCode::KeyI,   Button::Start);

    mappings.insert(KeyCode::KeyW,   Button::Up);
    mappings.insert(KeyCode::KeyS,   Button::Down);
    mappings.insert(KeyCode::KeyA,   Button::Left);
    mappings.insert(KeyCode::KeyD,   Button::Right);
    mappings.insert(KeyCode::ArrowUp,    Button::Up);
    mappings.insert(KeyCode::ArrowDown,  Button::Down);
    mappings.insert(KeyCode::ArrowLeft,  Button::Left);
    mappings.insert(KeyCode::ArrowRight, Button::Right);
    mappings
});

#[rustfmt::skip]
static JOY_2_KEYBOARD_MAPPINGS: LazyLock<HashMap<KeyCode, Button>> = LazyLock::new(|| {
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
});

static JOY_1_JOYPAD_MAPPINGS: LazyLock<HashMap<u32, Button>> = LazyLock::new(|| {
    let mut mappings = HashMap::new();
    mappings.insert(65824, Button::A);
    mappings.insert(65825, Button::B);
    mappings.insert(65830, Button::Select);
    mappings.insert(65831, Button::Start);
    mappings.insert(66080, Button::Up);
    mappings.insert(66081, Button::Down);
    mappings.insert(66082, Button::Left);
    mappings.insert(66083, Button::Right);
    mappings
});

/*
#[rustfmt::skip]
static JOY_2_JOYPAD_MAPPINGS: LazyLock<HashMap<u32, Button>> = LazyLock::new(|| {
    let mut mappings = HashMap::new();
    mappings.insert(VirtualKeyCode::Numpad0,        Button::A);
    mappings.insert(VirtualKeyCode::NumpadEnter,    Button::B);
    mappings.insert(VirtualKeyCode::NumpadSubtract, Button::Select);
    mappings.insert(VirtualKeyCode::NumpadAdd,      Button::Start);
    mappings.insert(VirtualKeyCode::Numpad8,        Button::Up);
    mappings.insert(VirtualKeyCode::Numpad5,        Button::Down);
    mappings.insert(VirtualKeyCode::Numpad4,        Button::Left);
    mappings.insert(VirtualKeyCode::Numpad6,        Button::Right);
    mappings
});
*/

pub struct PrimaryRenderer {
    paused: bool,
}

impl PrimaryRenderer {
    pub fn new() -> Self {
        Self {
            paused: false,
        }
    }
}

impl WindowRenderer for PrimaryRenderer {
    fn name(&self) -> String {
        "REZNEZ".to_string()
    }

    fn ui(&mut self, ctx: &Context, _world: &mut World) -> FlowControl {
        let mut result = FlowControl::CONTINUE;
        egui::TopBottomPanel::top("menubar_container").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Open").clicked() {
                        ui.close_menu();
                        let mut file_dialog = egui_file::FileDialog::open_file(None);
                        file_dialog.open();
                        result = FlowControl::spawn_window((
                            Box::new(LoadRomRenderer::new(file_dialog)) as Box<dyn WindowRenderer>,
                            Position::Physical(PhysicalPosition { x: 850, y: 360 }),
                            2,
                        ));
                    }
                    if ui.button("ROM Query").clicked() {
                        ui.close_menu();
                        let mut file_dialog = egui_file::FileDialog::select_folder(None);
                        file_dialog.open();
                        result = FlowControl::spawn_window((
                            Box::new(CartridgeQueryRenderer::new(file_dialog)) as Box<dyn WindowRenderer>,
                            Position::Physical(PhysicalPosition { x: 850, y: 360 }),
                            2,
                        ));
                    }
                });

                ui.menu_button("Settings", |ui| {
                    if ui.button("Display").clicked() {
                        ui.close_menu();
                        result = FlowControl::spawn_window((
                            Box::new(DisplaySettingsRenderer::new()) as Box<dyn WindowRenderer>,
                            Position::Physical(PhysicalPosition { x: 850, y: 360 }),
                            2,
                        ));
                    }
                });

                ui.menu_button("Debug Windows", |ui| {
                    if ui.button("Status").clicked() {
                        ui.close_menu();
                        result = FlowControl::spawn_window((
                            Box::new(StatusRenderer) as Box<dyn WindowRenderer>,
                            Position::Physical(PhysicalPosition { x: 850, y: 360 }),
                            2,
                        ));
                    }
                    if ui.button("Layers").clicked() {
                        ui.close_menu();
                        result = FlowControl::spawn_window((
                            Box::new(LayersRenderer::new()),
                            Position::Physical(PhysicalPosition { x: 850, y: 50 }),
                            1,
                        ));
                    }
                    if ui.button("Name Tables").clicked() {
                        ui.close_menu();
                        result = FlowControl::spawn_window((
                            Box::new(NameTableRenderer::new()),
                            Position::Physical(PhysicalPosition { x: 1400, y: 50 }),
                            1,
                        ));
                    }
                    if ui.button("Sprites").clicked() {
                        ui.close_menu();
                        result = FlowControl::spawn_window((
                            Box::new(SpritesRenderer::new()),
                            Position::Physical(PhysicalPosition { x: 1400, y: 660 }),
                            6,
                        ));
                    }
                    if ui.button("Pattern Tables").clicked() {
                        ui.close_menu();
                        result = FlowControl::spawn_window((
                            Box::new(PatternTableRenderer::new()),
                            Position::Physical(PhysicalPosition { x: 850, y: 660 }),
                            3,
                        ));
                    }
                    if ui.button("Pattern Sources").clicked() {
                        ui.close_menu();
                        result = FlowControl::spawn_window((
                            Box::new(PatternSourceRenderer::new()),
                            Position::Physical(PhysicalPosition { x: 600, y: 200 }),
                            1,
                        ));
                    }
                    if ui.button("Memory Viewer").clicked() {
                        ui.close_menu();
                        result = FlowControl::spawn_window((
                            Box::new(MemoryViewerRenderer),
                            Position::Physical(PhysicalPosition { x: 600, y: 200 }),
                            1,
                        ));
                    }
                    if ui.button("Cartridge Metadata").clicked() {
                        ui.close_menu();
                        result = FlowControl::spawn_window((
                            Box::new(CartridgeMetadataRenderer),
                            Position::Physical(PhysicalPosition { x: 600, y: 200 }),
                            2,
                        ));
                    }
                })
            });
        });

        result
    }

    fn render(&mut self, world: &mut World, pixels: &mut Pixels) {
        if self.paused {
            return;
        }

        let display_frame = |frame: &Frame, mask, _frame_index| {
            frame.copy_to_rgba_buffer(mask, pixels.frame_mut().try_into().unwrap());
        };

        if let Some(nes) = &mut world.nes {
            execute_frame(
                nes,
                &world.config,
                &events(&world.input, &mut world.gilrs, world.active_gamepad_id),
                display_frame,
            );
        }
    }

    fn toggle_pause(&mut self) {
        self.paused = !self.paused;
    }

    fn width(&self) -> usize {
        PixelColumn::COLUMN_COUNT
    }

    fn height(&self) -> usize {
        PixelRow::ROW_COUNT
    }
}

fn events(input: &WinitInputHelper, gilrs: &mut gilrs::Gilrs, active_gamepad_id: Option<GamepadId>) -> Events {
    let mut joypad1_button_statuses = BTreeMap::new();
    let mut joypad2_button_statuses = BTreeMap::new();

    while let Some(gilrs::Event { id, event, .. }) = gilrs.next_event() {
        assert_eq!(Some(id), active_gamepad_id);
        match event {
            gilrs::EventType::ButtonPressed(_, code) => {
                if let Some(button) = JOY_1_JOYPAD_MAPPINGS.get(&code.into_u32()) {
                    joypad1_button_statuses.insert(*button, ButtonStatus::Pressed);
                }
            }
            gilrs::EventType::ButtonReleased(_, code) => {
                if let Some(button) = JOY_1_JOYPAD_MAPPINGS.get(&code.into_u32()) {
                    joypad1_button_statuses.insert(*button, ButtonStatus::Unpressed);
                }
            }
            _ => {}
        }
    }

    for (&key, &button) in JOY_1_KEYBOARD_MAPPINGS.iter() {
        if input.key_pressed(key) {
            joypad1_button_statuses.insert(button, ButtonStatus::Pressed);
        } else if input.key_released(key) {
            joypad1_button_statuses.insert(button, ButtonStatus::Unpressed);
        };
    }

    for (&key, &button) in JOY_2_KEYBOARD_MAPPINGS.iter() {
        if input.key_pressed(key) {
            joypad2_button_statuses.insert(button, ButtonStatus::Pressed);
        } else if input.key_released(key) {
            joypad2_button_statuses.insert(button, ButtonStatus::Unpressed);
        };
    }

    Events {
        // Quit-handling is done by winit.
        should_quit: false,
        joypad1_button_statuses,
        joypad2_button_statuses,
    }
}
