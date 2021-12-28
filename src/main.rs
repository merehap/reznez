#![feature(array_chunks)]
#![feature(const_option)]
#![feature(destructuring_assignment)]
#![allow(dead_code)]
#![allow(clippy::module_inception)]
#![allow(clippy::new_without_default)]

mod cartridge;
mod config;
mod controller;
mod cpu;
mod gui;
mod ppu;
mod mapper;
pub mod nes;
mod util;

use std::path::Path;

use crate::config::Config;
use crate::gui::sdl_gui::SdlGui;
use crate::nes::Nes;

fn main() {
    let config = Config::default(Path::new("roms/Donkey Kong.nes"));
    let mut nes = Nes::new(config);
    let mut gui = SdlGui::initialize();
    loop {
        nes.step_frame(&mut gui);
    }
}
