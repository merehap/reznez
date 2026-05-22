use winit_input_helper::WinitInputHelper;

use crate::{config::Config, nes::Nes};
use crate::gui::gui::Events;

pub struct World {
    pub nes: Option<Nes>,
    pub config: Config,
    pub input: WinitInputHelper,
    pub gilrs: gilrs::Gilrs,
    pub active_gamepad_id: Option<gilrs::GamepadId>,
    pub events: Events,
}
