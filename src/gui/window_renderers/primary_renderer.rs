use std::path::Path;

use egui::{include_image, vec2, Align2, CentralPanel, Context, Frame as EguiFrame, Image};
use egui_file::FileDialog;
use log::error;
use pixels::Pixels;
pub use winit::dpi::{PhysicalPosition, Position};

use crate::cartridge::header_db::HeaderDb;
use crate::config::Config;
use crate::gui::gui::{execute_frame, Events};
pub use crate::gui::window_renderer::{FlowControl, WindowRenderer};
use crate::nes::Nes;
use crate::gui::window_renderers::audio_visualizer::AudioVisualizer;
use crate::gui::window_renderers::cartridge_metadata_renderer::CartridgeMetadataRenderer;
use crate::gui::window_renderers::cartridge_query_renderer::{CartridgeQueryRenderer};
use crate::gui::window_renderers::controls_renderer::ControlsRenderer;
use crate::gui::window_renderers::display_settings_renderer::DisplaySettingsRenderer;
use crate::gui::window_renderers::layers_renderer::LayersRenderer;
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
    file_dialog: FileDialog,
    load_error: Option<String>,
    cartridge_query_dialog: FileDialog,
}

impl PrimaryRenderer {
    pub fn new() -> Self {
        let nes_file_filter = Box::new(|path: &Path| {
            path.extension()
                .and_then(|extension| extension.to_str())
                .is_some_and(|extension| extension.eq_ignore_ascii_case("nes"))
        });

        let file_dialog = FileDialog::open_file()
            .show_files_filter(nes_file_filter)
            .anchor(Align2::CENTER_CENTER, vec2(0.0, 0.0));
        let cartridge_query_dialog = FileDialog::select_folder()
            .anchor(Align2::CENTER_CENTER, vec2(0.0, 0.0));

        Self {
            paused: false,
            file_dialog,
            load_error: None,
            cartridge_query_dialog,
        }
    }
}

impl WindowRenderer for PrimaryRenderer {
    fn name(&self) -> String {
        "REZNEZ".to_string()
    }

    fn ui(&mut self, ctx: &Context, world: &mut World) -> FlowControl {
        let mut result = FlowControl::CONTINUE;
        egui::TopBottomPanel::top("menubar_container").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Open").clicked() {
                        ui.close_menu();
                        self.load_error = None;
                        self.file_dialog.open();
                    }

                    if ui.button("ROM Query").clicked() {
                        ui.close_menu();
                        self.cartridge_query_dialog.open();
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

                ui.menu_button("Help", |ui| {
                    if ui.button("Controls").clicked() {
                        ui.close_menu();
                        result = FlowControl::spawn_window((
                            Box::new(ControlsRenderer) as Box<dyn WindowRenderer>,
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

        if world.nes.is_none() {
            CentralPanel::default()
                .frame(EguiFrame::none())
                .show(ctx, |ui| {
                    let available_size = ui.available_size();
                    ui.add(
                         Image::new(include_image!("../assets/reznez_splash.svg"))
                            .fit_to_exact_size(available_size),
                    );
                });
        }

        self.file_dialog.show(ctx);
        self.cartridge_query_dialog.show(ctx);

        if let Some(load_error) = &self.load_error {
            let mut choose_another_file = false;
            CentralPanel::default().show(ctx, |ui| {
                ui.colored_label(egui::Color32::RED, load_error);
                if ui.button("Choose another file").clicked() {
                    choose_another_file = true;
                }
            });

            if choose_another_file {
                self.load_error = None;
                self.file_dialog.open();
            }
        }


        if self.file_dialog.selected() {
            if let Some(rom_path) = self.file_dialog.path() && !rom_path.is_dir() {
                let header_db = HeaderDb::load();
                match load_nes(&header_db, &world.config, rom_path) {
                    Ok(nes) => {
                        world.nes = Some(nes);
                    }
                    Err(err) => {
                        error!("Failed to load ROM {}. {err}", rom_path.to_string_lossy());
                        self.load_error = Some(format!("Failed to load ROM.\nDetails: {err}"));
                        let current_directory = self.file_dialog.directory().to_owned();
                        self.file_dialog.set_path(current_directory);
                    }
                }
            }
        }

        if self.cartridge_query_dialog.selected() {
            result = FlowControl::spawn_window((
                Box::new(CartridgeQueryRenderer::new(self.cartridge_query_dialog.directory())) as Box<dyn WindowRenderer>,
                Position::Physical(PhysicalPosition { x: 50, y: 50 }),
                1,
            ));
        }
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

fn load_nes(header_db: &HeaderDb, config: &Config, rom_path: &Path) -> Result<Nes, String> {
    let cartridge = Nes::load_cartridge(rom_path)?;
    Nes::new(header_db, config, &cartridge)
}