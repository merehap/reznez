use std::collections::BTreeMap;

use crate::controller::joypad::{Button, ButtonStatus};
use crate::ppu::render::frame::Frame;

pub trait Gui {
    fn initialize() -> Self where Self: Sized;
    fn events(&mut self) -> Events;
    fn frame_mut(&mut self) -> &mut Frame;
    fn display_frame(&mut self, frame_index: u64);
}

pub struct Events {
    pub should_quit: bool,
    pub joypad_1_button_statuses: BTreeMap<Button, ButtonStatus>,
    pub joypad_2_button_statuses: BTreeMap<Button, ButtonStatus>,
}
