use egui::Context;
use pixels::Pixels;

use crate::gui::window_renderer::{FlowControl, WindowRenderer};
use crate::gui::world::World;

pub struct ControlsRenderer;

impl ControlsRenderer {
    const WIDTH: usize = 220;
    const HEIGHT: usize = 260;
}

impl WindowRenderer for ControlsRenderer {
    fn name(&self) -> String {
        "Controls".to_string()
    }

    fn ui(&mut self, ctx: &Context, _world: &mut World) -> FlowControl {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Controls");
            ui.separator();

            ui.label("Player 1:");
            egui::Grid::new("player_1_controls")
                .num_columns(2)
                .spacing([40.0, 4.0])
                .striped(true)
                .show(ui, |ui| {
                    ui.label("A");
                    ui.label("K");
                    ui.end_row();
                    ui.label("B");
                    ui.label("J");
                    ui.end_row();
                    ui.label("Start");
                    ui.label("I");
                    ui.end_row();
                    ui.label("Select");
                    ui.label("U");
                    ui.end_row();
                    ui.label("D-pad");
                    ui.label("WASD or arrow keys");
                    ui.end_row();
                });

            ui.add_space(10.0);
            ui.label("Gamepad:");
            ui.label("One physical gamepad is currently supported for Player 1.");

            ui.add_space(10.0);
            ui.label("Player 2:");
            egui::Grid::new("player_2_controls")
                .num_columns(2)
                .spacing([40.0, 4.0])
                .striped(true)
                .show(ui, |ui| {
                    ui.label("A");
                    ui.label("Numpad 0");
                    ui.end_row();
                    ui.label("B");
                    ui.label("Numpad Enter");
                    ui.end_row();
                    ui.label("Start");
                    ui.label("Numpad +");
                    ui.end_row();
                    ui.label("Select");
                    ui.label("Numpad -");
                    ui.end_row();
                    ui.label("D-pad");
                    ui.label("Numpad 8, 5, 4, 6");
                    ui.end_row();
                });

            ui.add_space(10.0);
            ui.label("Shortcuts:");
            egui::Grid::new("shortcuts")
                .num_columns(2)
                .spacing([40.0, 4.0])
                .striped(true)
                .show(ui, |ui| {
                    ui.label("Pause / Resume");
                    ui.label("Esc or P");
                    ui.end_row();
                    ui.label("Reload Game");
                    ui.label("F12");
                    ui.end_row();
                });
        });

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
