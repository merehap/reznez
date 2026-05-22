pub use egui::Context;
use pixels::Pixels;
pub use winit::dpi::{PhysicalPosition, Position};

use crate::gui::gui::{execute_frame, Events};
pub use crate::gui::window_renderer::{FlowControl, WindowRenderer};
use crate::gui::window_renderers::audio_visualizer::AudioVisualizer;
use crate::gui::window_renderers::cartridge_metadata_renderer::CartridgeMetadataRenderer;
use crate::gui::window_renderers::cartridge_query_renderer::CartridgeQueryPopupRenderer;
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

pub struct PrimaryRenderer {
    pub paused: bool,
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
                            Box::new(CartridgeQueryPopupRenderer::new(file_dialog)) as Box<dyn WindowRenderer>,
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
                    if ui.button("Audio Visualizer").clicked() {
                        ui.close_menu();
                        result = FlowControl::spawn_window((
                            Box::new(AudioVisualizer::new()),
                            Position::Physical(PhysicalPosition { x: 600, y: 200 }),
                            2,
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
                std::mem::replace(&mut world.events, Events::none()),
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
