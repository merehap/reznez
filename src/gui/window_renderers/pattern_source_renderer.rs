
use egui::Context;
use pixels::Pixels;

use crate::gui::window_renderer::{FlowControl, WindowRenderer};
use crate::gui::world::World;
use crate::ppu::palette::rgb::Rgb;
use crate::ppu::pixel_index::{PixelColumn, PixelRow};
use crate::ppu::render::frame::DebugBuffer;

pub struct PatternSourceRenderer {
    buffer: DebugBuffer<{ PixelColumn::COLUMN_COUNT }, { PixelRow::ROW_COUNT }>,
}

impl PatternSourceRenderer {
    pub fn new() -> Self {
        Self {
            buffer: DebugBuffer::new(Rgb::WHITE),
        }
    }
}

impl WindowRenderer for PatternSourceRenderer {
    fn name(&self) -> String {
        "Pattern Source".to_string()
    }

    fn ui(&mut self, _ctx: &Context, _world: &mut World) -> FlowControl {
        FlowControl::CONTINUE
    }

    fn render(&mut self, world: &mut World, pixels: &mut Pixels) {
        let Some(nes) = &world.nes else {
            return;
        };

        self.buffer.place_frame(0, 0, nes.ppu().pattern_source_frame());
        self.buffer.copy_to_rgba_buffer(pixels.frame_mut());
    }

    fn width(&self) -> usize {
        PixelColumn::COLUMN_COUNT
    }

    fn height(&self) -> usize {
        PixelRow::ROW_COUNT
    }
}