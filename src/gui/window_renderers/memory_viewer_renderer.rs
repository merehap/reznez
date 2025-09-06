use egui::Context;
use pixels::Pixels;

use crate::gui::window_renderer::{FlowControl, WindowRenderer};
use crate::gui::world::World;
use crate::mapper::CpuAddress;

pub struct MemoryViewerRenderer;

impl MemoryViewerRenderer {
    const WIDTH: usize = 700;
    const HEIGHT: usize = 400;
}

impl WindowRenderer for MemoryViewerRenderer {
    fn name(&self) -> String {
        "Memory Viewer".to_string()
    }

    fn ui(&mut self, ctx: &Context, world: &mut World) -> FlowControl {
        let Some(nes) = &world.nes else {
            return FlowControl::CONTINUE;
        };

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                egui::Grid::new("my_grid")
                    .num_columns(16)
                    .spacing([0.0, 0.0])
                    .striped(true)
                    .show(ui, |ui| {
                        for mem_index in 0..=u16::MAX {
                            let value = nes.mapper().cpu_peek(nes.memory(), CpuAddress::new(mem_index)).resolve(nes.memory().cpu_data_bus);
                            let _ = ui.button(format!("{value:02X}"));
                            if mem_index % 0x10 == 0x0F {
                                ui.end_row();
                            }
                        }
                    });
            })
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