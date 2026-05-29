use std::path::Path;

use egui::Context;
use egui_file::FileDialog;
use log::error;
use pixels::Pixels;

use crate::cartridge::header_db::HeaderDb;
use crate::config::Config;
use crate::gui::window_renderer::{FlowControl, WindowRenderer};
use crate::gui::world::World;
use crate::nes::Nes;

pub struct LoadRomRenderer {
    file_dialog: FileDialog,
    load_error: Option<String>,
}

impl LoadRomRenderer {
    const WIDTH: usize = 300;
    const HEIGHT: usize = 300;

    pub fn new(file_dialog: FileDialog) -> Self {
        Self {
            file_dialog,
            load_error: None,
        }
    }
}

impl WindowRenderer for LoadRomRenderer {
    fn name(&self) -> String {
        "Load ROM".to_string()
    }

    fn ui(&mut self, ctx: &Context, world: &mut World) -> FlowControl {
        let mut result = FlowControl::CONTINUE;
        let header_db = HeaderDb::load();
        egui::CentralPanel::default().show(ctx, |ui| {
            self.file_dialog.show(ctx);
            if let Some(load_error) = &self.load_error {
                ui.colored_label(egui::Color32::RED, load_error);
            }

            if let Some(rom_path) = self.file_dialog.path() && !rom_path.is_dir() {
                match load_nes(&header_db, &world.config, rom_path) {
                    Ok(nes) => {
                        world.nes = Some(nes);
                        result = FlowControl::CLOSE;
                    }
                    Err(err) => {
                        error!("Failed to load ROM {}. {err}", rom_path.to_string_lossy());
                        self.load_error = Some(format!("Failed to load ROM.\nDetails: {err}"));
                        let current_directory = self.file_dialog.directory().to_owned();
                        self.file_dialog.set_path(current_directory);

                    }
                }
            }

        });

        result
    }

    fn render(&mut self, _world: &mut World, _pixels: &mut Pixels) {
        // Do nothing yet.
    }

    fn width(&self) -> usize {
        Self::WIDTH
    }

    fn height(&self) -> usize {
        Self::HEIGHT
    }
}

fn load_nes(header_db: &HeaderDb, config: &Config, rom_path: &Path) -> Result<Nes, String> {
    let cartridge = Nes::load_cartridge(rom_path)?;
    Nes::new(header_db, config, &cartridge)
}