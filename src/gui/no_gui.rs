use crate::config::Config;
use crate::gui::gui::Gui;
use crate::nes::Nes;

pub struct NoGui;

impl NoGui {
    pub fn new() -> NoGui {
        NoGui
    }
}

impl Gui for NoGui {
    fn run(&mut self, mut nes: Nes, _config: Config) {
        loop {
            nes.step_frame();
        }
    }
}
