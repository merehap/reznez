use std::collections::BTreeMap;

use crate::controller::joypad::{Button, ButtonStatus};
use crate::ppu::render::frame::Frame;

pub trait Gui {
    fn events(&mut self) -> Events;
    fn frame_mut(&mut self) -> &mut Frame;
    fn display_frame(&mut self, frame_index: u64);
}

pub struct Events {
    pub should_quit: bool,
    pub joypad1_button_statuses: BTreeMap<Button, ButtonStatus>,
    pub joypad2_button_statuses: BTreeMap<Button, ButtonStatus>,
}
