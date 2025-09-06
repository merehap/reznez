use egui::Context;
use pixels::Pixels;
use winit::dpi::Position;

use crate::gui::world::World;

pub trait WindowRenderer {
    fn name(&self) -> String;
    fn ui(&mut self, ctx: &Context, world: &mut World) -> FlowControl;
    fn render(&mut self, world: &mut World, pixels: &mut Pixels);
    fn toggle_pause(&mut self) {}
    fn width(&self) -> usize;
    fn height(&self) -> usize;
}

pub type WindowArgs = (Box<dyn WindowRenderer>, Position, u64);

pub struct FlowControl {
    pub window_args: Option<WindowArgs>,
    pub should_close_window: bool,
}

impl FlowControl {
    pub const CONTINUE: Self = Self {
        window_args: None,
        should_close_window: false,
    };
    pub const CLOSE: Self = Self {
        window_args: None,
        should_close_window: true,
    };

    pub fn spawn_window(window_args: WindowArgs) -> Self {
        Self {
            window_args: Some(window_args),
            should_close_window: false,
        }
    }
}