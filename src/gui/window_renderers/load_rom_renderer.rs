use egui::Context;
use egui_file::FileDialog;
use pixels::Pixels;

use crate::cartridge::header_db::HeaderDb;
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
                let cartridge = Nes::load_cartridge(rom_path);
                world.nes = Some(Nes::new(&header_db, &world.config, cartridge));
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
