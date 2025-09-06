use egui::Context;
use pixels::Pixels;

use crate::gui::debug_screens::pattern_table::PatternTable;
use crate::gui::window_renderer::{FlowControl, WindowRenderer};
use crate::gui::world::World;
use crate::ppu::palette::rgb::Rgb;
use crate::ppu::render::frame::DebugBuffer;

pub struct SpritesRenderer {
    buffer: DebugBuffer<{ SpritesRenderer::WIDTH }, { SpritesRenderer::HEIGHT }>,
}

impl SpritesRenderer {
    const WIDTH: usize = 8 * (8 + 1);
    const HEIGHT: usize = 8 * (8 + 1);

    pub fn new() -> SpritesRenderer {
        SpritesRenderer { buffer: DebugBuffer::new(Rgb::WHITE) }
    }
}

impl WindowRenderer for SpritesRenderer {
    fn name(&self) -> String {
        "Sprites".to_string()
    }

    fn ui(&mut self, _ctx: &Context, _world: &mut World) -> FlowControl {
        FlowControl::CONTINUE
    }

    fn render(&mut self, world: &mut World, pixels: &mut Pixels) {
        let Some(nes) = &world.nes else {
            return;
        };

        let sprites = nes.memory().oam().sprites();
        let mem = nes.memory();

        for (index, sprite) in sprites.iter().enumerate() {
            let tile = sprite.render_normal_height(&PatternTable::sprite_side(mem), &mem.palette_table());
            self.buffer.place_tile(
                (8 + 1) * (index % 8),
                (8 + 1) * (index / 8),
                &tile,
            );
        }

        self.buffer.copy_to_rgba_buffer(pixels.frame_mut());
    }

    fn width(&self) -> usize {
        Self::WIDTH
    }

    fn height(&self) -> usize {
        Self::HEIGHT
    }
}