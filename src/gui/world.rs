use crate::{config::Config, nes::Nes};
use crate::gui::gui::Events;

pub struct World {
    pub nes: Option<Nes>,
    pub config: Config,
    pub events: Events,
}
