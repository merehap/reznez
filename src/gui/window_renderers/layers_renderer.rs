use egui::Context;
use pixels::Pixels;

use crate::gui::window_renderer::{FlowControl, WindowRenderer};
use crate::gui::world::World;
use crate::ppu::palette::rgb::Rgb;
use crate::ppu::render::frame::{DebugBuffer, Frame};

const TOP_MENU_BAR_HEIGHT: usize = 24;

pub struct LayersRenderer {
    frame: Frame,
    buffer: DebugBuffer<{ LayersRenderer::WIDTH }, { LayersRenderer::HEIGHT }>,
}

impl LayersRenderer {
    const WIDTH: usize = 517;
    const HEIGHT: usize = 485 + TOP_MENU_BAR_HEIGHT;

    pub fn new() -> LayersRenderer {
        LayersRenderer {
            frame: Frame::new(),
            buffer: DebugBuffer::new(Rgb::WHITE),
        }
    }
}

impl WindowRenderer for LayersRenderer {
    fn name(&self) -> String {
        "Layers".to_string()
    }

    fn ui(&mut self, _ctx: &Context, _world: &mut World) -> FlowControl {
        FlowControl::CONTINUE
    }

    fn render(&mut self, world: &mut World, pixels: &mut Pixels) {
        let Some(nes) = &world.nes else {
            return;
        };

        self.buffer
            .place_frame(0, TOP_MENU_BAR_HEIGHT, nes.frame());
        self.buffer.place_frame(
            261,
            TOP_MENU_BAR_HEIGHT,
            &nes.frame().to_background_only(),
        );

        let bus = nes.bus();

        self.frame.clear();
        bus.oam.only_front_sprites().render(bus, &mut self.frame);
        self.buffer
            .place_frame(0, 245 + TOP_MENU_BAR_HEIGHT, &self.frame);

        self.frame.clear();
        bus.oam.only_back_sprites().render(bus, &mut self.frame);
        self.buffer
            .place_frame(261, 245 + TOP_MENU_BAR_HEIGHT, &self.frame);

        self.buffer.copy_to_rgba_buffer(pixels.frame_mut());
    }

    fn width(&self) -> usize {
        Self::WIDTH
    }

    fn height(&self) -> usize {
        Self::HEIGHT
    }
}