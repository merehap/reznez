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
}

impl LoadRomRenderer {
    const WIDTH: usize = 300;
    const HEIGHT: usize = 300;

    pub fn new(file_dialog: FileDialog) -> Self {
        Self {
            file_dialog,
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
        egui::CentralPanel::default().show(ctx, |_ui| {
            self.file_dialog.show(ctx);
            if let Some(rom_path) = self.file_dialog.path() && !rom_path.is_dir() {
                result = FlowControl::CLOSE;

                world.nes = match load_nes(&header_db, &world.config, rom_path) {
                    Ok(nes) => Some(nes),
                    Err(err) => {
                        error!("Failed to load ROM {}. {err}", rom_path.to_string_lossy());
                        None
                    }
                };
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