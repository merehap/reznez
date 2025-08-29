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
    fn run(&mut self, nes: Option<Nes>, _config: Config) {
        let mut nes = nes.expect("ROM to be specified when nogui mode is specified.");
        loop {
            nes.step_frame();
        }
    }
}
