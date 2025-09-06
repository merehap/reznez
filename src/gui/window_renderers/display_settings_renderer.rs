use egui::Context;
use pixels::Pixels;

use crate::gui::window_renderer::{FlowControl, WindowRenderer};
use crate::gui::world::World;

pub struct DisplaySettingsRenderer;

impl DisplaySettingsRenderer {
    const WIDTH: usize = 300;
    const HEIGHT: usize = 300;

    pub fn new() -> Self {
        Self
    }
}

impl WindowRenderer for DisplaySettingsRenderer {
    fn name(&self) -> String {
        "Display Settings".to_string()
    }

    fn ui(&mut self, ctx: &Context, world: &mut World) -> FlowControl {
        if let Some(nes) = &mut world.nes {
            egui::CentralPanel::default().show(ctx, |ui| {
                egui::Grid::new("my_grid")
                    .num_columns(2)
                    .spacing([40.0, 4.0])
                    .striped(true)
                    .show(ui, |ui| {
                        ui.checkbox(nes.frame_mut().show_overscan_mut(), "Show overscan?");
                        ui.end_row();
                    });
            });
        }

        FlowControl::CONTINUE
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
