use std::collections::BTreeMap;

use crate::gui::gui::{Gui, Events};
use crate::ppu::render::frame::Frame;

pub struct NoGui {
    frame: Frame,
}

impl Gui for NoGui {
    fn initialize() -> NoGui {
        NoGui {
            frame: Frame::new(),
        }
    }

    #[inline]
    fn events(&mut self) -> Events {
        Events {
            should_quit: false,
            joypad1_button_statuses: BTreeMap::new(),
            joypad2_button_statuses: BTreeMap::new(),
        }
    }

    fn frame_mut(&mut self) -> &mut Frame {
        &mut self.frame
    }

    fn display_frame(&mut self, _frame_index: u64) {
        // Do nothing.
    }
}
