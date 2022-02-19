use std::collections::BTreeMap;

use crate::gui::gui::{Gui, Events};
use crate::ppu::render::frame::Frame;

pub struct NoGui;

impl NoGui {
    pub fn new() -> NoGui {
        NoGui
    }
}

impl Gui for NoGui {
    #[inline]
    fn events(&mut self) -> Events {
        Events {
            should_quit: false,
            joypad1_button_statuses: BTreeMap::new(),
            joypad2_button_statuses: BTreeMap::new(),
        }
    }

    fn display_frame(&mut self, _: &Frame, _frame_index: u64) {
        // Do nothing.
    }
}
