use crate::gui::gui::Gui;
use crate::nes::Nes;

pub struct NoGui;

impl Gui for NoGui {
    fn run(&mut self, nes: Option<Nes>) {
        let mut nes = nes.expect("ROM to be specified when nogui mode is specified.");
        loop {
            nes.step_frame();
        }
    }
}
