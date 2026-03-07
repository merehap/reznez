use egui::Context;
use egui_plot::{GridMark, Line, Plot};
use pixels::Pixels;

use crate::gui::window_renderer::{FlowControl, WindowRenderer};
use crate::gui::world::World;

const SAMPLE_COUNT: u32 = 1000;

pub struct AudioVisualizer {
    buffer: Vec<[f64; 2]>,
}

impl AudioVisualizer {
    const WIDTH: usize = 500;
    const HEIGHT: usize = 800;

    pub fn new() -> Self {
        let mut buffer = vec![[0.0, 0.0]; SAMPLE_COUNT as usize];
        for i in 0..SAMPLE_COUNT as usize {
            buffer[i][0] = i as f64 * 0.1;
        }

        Self { buffer }
    }
}

impl WindowRenderer for AudioVisualizer {
    fn name(&self) -> String {
        "Audio Visualizer".to_string()
    }

    fn ui(&mut self, ctx: &Context, world: &mut World) -> FlowControl {
        let Some(nes) = &world.nes else {
            return FlowControl::CONTINUE;
        };

        egui::CentralPanel::default().show(ctx, |ui| {
            nes.bus().apu_regs.pulse1_volumes.clone_to(&mut self.buffer);
            // Stop cloning once egui is upgraded.
            let line = Line::new(self.buffer.clone());
            Plot::new("my_plot")
                .view_aspect(6.0)
                .allow_scroll(false)
                .allow_zoom(false)
                .allow_drag(false)
                .clamp_grid(true)
                .show_x(false)
                .x_grid_spacer(|_| vec![])
                .y_grid_spacer(|_| vec![
                    GridMark { value:  0.0, step_size: 5.0 },
                    GridMark { value:  5.0, step_size: 5.0 },
                    GridMark { value: 10.0, step_size: 5.0 },
                    GridMark { value: 15.0, step_size: 5.0 },
                ])
                .show(ui, |plot_ui| {
                    plot_ui.line(line);
                    // Force the y-axis to always have a max value of 15.
                    plot_ui.line(Line::new(vec![[0.0, 15.0]]));
                });

            nes.bus().apu_regs.pulse2_volumes.clone_to(&mut self.buffer);
            let line = Line::new(self.buffer.clone());
            Plot::new("my_plot")
                .view_aspect(6.0)
                .allow_scroll(false)
                .allow_zoom(false)
                .allow_drag(false)
                .clamp_grid(true)
                .show_x(false)
                .x_grid_spacer(|_| vec![])
                .y_grid_spacer(|_| vec![
                    GridMark { value:  5.0, step_size: 5.0 },
                    GridMark { value: 10.0, step_size: 5.0 },
                    GridMark { value: 15.0, step_size: 5.0 },
                ])
                .show(ui, |plot_ui| {
                    plot_ui.line(line);
                    plot_ui.line(Line::new(vec![[0.0, 15.0]]));
                });

                nes.bus().apu_regs.triangle_volumes.clone_to(&mut self.buffer);
                let line = Line::new(self.buffer.clone());
                Plot::new("my_plot")
                    .view_aspect(6.0)
                    .allow_scroll(false)
                    .allow_zoom(false)
                    .allow_drag(false)
                    .clamp_grid(true)
                    .show_x(false)
                    .x_grid_spacer(|_| vec![])
                    .y_grid_spacer(|_| vec![
                        GridMark { value:  5.0, step_size: 5.0 },
                        GridMark { value: 10.0, step_size: 5.0 },
                        GridMark { value: 15.0, step_size: 5.0 },
                    ])
                    .show(ui, |plot_ui| {
                        plot_ui.line(line);
                        plot_ui.line(Line::new(vec![[0.0, 15.0]]));
                    });

                nes.bus().apu_regs.noise_volumes.clone_to(&mut self.buffer);
                let line = Line::new(self.buffer.clone());
                Plot::new("my_plot")
                    .view_aspect(6.0)
                    .allow_scroll(false)
                    .allow_zoom(false)
                    .allow_drag(false)
                    .clamp_grid(true)
                    .show_x(false)
                    .x_grid_spacer(|_| vec![])
                    .y_grid_spacer(|_| vec![
                        GridMark { value:  5.0, step_size: 5.0 },
                        GridMark { value: 10.0, step_size: 5.0 },
                        GridMark { value: 15.0, step_size: 5.0 },
                    ])
                    .show(ui, |plot_ui| {
                        plot_ui.line(line);
                        plot_ui.line(Line::new(vec![[0.0, 15.0]]));
                    });

                nes.bus().apu_regs.dmc_volumes.clone_to(&mut self.buffer);
                let line = Line::new(self.buffer.clone());
                Plot::new("my_plot")
                    .view_aspect(6.0)
                    .allow_scroll(false)
                    .allow_zoom(false)
                    .allow_drag(false)
                    .clamp_grid(true)
                    .show_x(false)
                    .x_grid_spacer(|_| vec![])
                    .y_grid_spacer(|_| vec![
                        GridMark { value:  5.0, step_size: 5.0 },
                        GridMark { value: 10.0, step_size: 5.0 },
                        GridMark { value: 15.0, step_size: 5.0 },
                    ])
                    .show(ui, |plot_ui| {
                        plot_ui.line(line);
                        plot_ui.line(Line::new(vec![[0.0, 15.0]]));
                    });

                nes.bus().apu_regs.mixed_values.clone_to(&mut self.buffer);
                let line = Line::new(self.buffer.clone());
                Plot::new("my_plot")
                    .view_aspect(6.0)
                    .allow_scroll(false)
                    .allow_zoom(false)
                    .allow_drag(false)
                    .clamp_grid(true)
                    .show_x(false)
                    .x_grid_spacer(|_| vec![])
                    .y_grid_spacer(|_| vec![
                        GridMark { value: 0.0, step_size: 0.5 },
                        GridMark { value: 0.5, step_size: 0.5 },
                        GridMark { value: 1.0, step_size: 0.5 },
                    ])
                    .show(ui, |plot_ui| {
                        plot_ui.line(line);
                        plot_ui.line(Line::new(vec![[0.0, 1.0]]));
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