use egui::Context;
use pixels::Pixels;

use crate::gui::debug_screens::name_table::NameTable;
use crate::gui::debug_screens::pattern_table::PatternTable;
use crate::gui::window_renderer::{FlowControl, WindowRenderer};
use crate::gui::world::World;
use crate::mapper::NameTableQuadrant;
use crate::ppu::palette::rgb::Rgb;
use crate::ppu::render::frame::{DebugBuffer, Frame};

pub struct NameTableRenderer {
    frame: Frame,
    buffer: DebugBuffer<{ NameTableRenderer::WIDTH }, { NameTableRenderer::HEIGHT }>,
}

impl NameTableRenderer {
    const WIDTH: usize = 2 * 256 + 2;
    const HEIGHT: usize = 2 * 240 + 2;

    pub fn new() -> NameTableRenderer {
        NameTableRenderer {
            frame: Frame::new(),
            buffer: DebugBuffer::new(Rgb::WHITE),
        }
    }
}

impl WindowRenderer for NameTableRenderer {
    fn name(&self) -> String {
        "Name Tables".to_string()
    }

    fn ui(&mut self, _ctx: &Context, _world: &mut World) -> FlowControl {
        FlowControl::CONTINUE
    }

    #[rustfmt::skip]
    fn render(&mut self, world: &mut World, pixels: &mut Pixels) {
        let Some(nes) = &world.nes else {
            return;
        };

        let x = usize::from(nes.memory().ppu_regs.x_scroll().to_u8());
        let y = usize::from(nes.memory().ppu_regs.y_scroll().to_u8());
        let mem = &mut nes.memory();

        let width = NameTableRenderer::WIDTH;
        let height = NameTableRenderer::HEIGHT;
        // Clear any junk out of the outer border.
        self.buffer.place_wrapping_horizontal_line(0, 0, width, Rgb::new(255, 255, 255));
        self.buffer.place_wrapping_horizontal_line(height, 0, width, Rgb::new(255, 255, 255));
        self.buffer.place_wrapping_vertical_line(0, 0, height, Rgb::new(255, 255, 255));
        self.buffer.place_wrapping_vertical_line(width, 0, height, Rgb::new(255, 255, 255));

        self.frame.set_universal_background_rgb(mem.palette_table().universal_background_rgb());
        let background_table = PatternTable::background_side(mem);

        NameTable::new(mem.raw_name_table(NameTableQuadrant::TopLeft))
            .render(&background_table, &mem.palette_table(), &mut self.frame);
        self.buffer.place_frame(1, 1, &self.frame);
        NameTable::new(mem.raw_name_table(NameTableQuadrant::TopRight))
            .render(&background_table, &mem.palette_table(), &mut self.frame);
        self.buffer.place_frame(257, 1, &self.frame);
        NameTable::new(mem.raw_name_table(NameTableQuadrant::BottomLeft))
            .render(&background_table, &mem.palette_table(), &mut self.frame);
        self.buffer.place_frame(1, 241, &self.frame);
        NameTable::new(mem.raw_name_table(NameTableQuadrant::BottomRight))
            .render(&background_table, &mem.palette_table(), &mut self.frame);
        self.buffer.place_frame(257, 241, &self.frame);

        self.buffer.place_wrapping_horizontal_line(y, x, x + 257, Rgb::new(255, 0, 0));
        self.buffer.place_wrapping_horizontal_line(y + 241, x, x + 257, Rgb::new(255, 0, 0));
        self.buffer.place_wrapping_vertical_line(x, y, y + 241, Rgb::new(255, 0, 0));
        self.buffer.place_wrapping_vertical_line(x + 257, y, y + 241, Rgb::new(255, 0, 0));

        self.buffer.copy_to_rgba_buffer(pixels.frame_mut());
    }

    fn width(&self) -> usize {
        Self::WIDTH
    }

    fn height(&self) -> usize {
        Self::HEIGHT
    }
}