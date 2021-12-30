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

use std::env;
use std::path::{Path, PathBuf};

use structopt::StructOpt;

use crate::config::Config;
use crate::gui::gui::Gui;
use crate::gui::frame_dump_gui::FrameDumpGui;
use crate::gui::sdl_gui::SdlGui;
use crate::nes::Nes;

fn main() {
    let opt = Opt::from_args();
    println!("{:?}", opt);

    let config = Config::default(Path::new(&opt.rom_path));
    let mut nes = Nes::new(config);
    let mut gui = match opt.gui.as_str() {
        "sdl" => Box::new(SdlGui::initialize()) as Box<dyn Gui>,
        "framedump" => Box::new(FrameDumpGui::initialize()) as Box<dyn Gui>,
        _ => panic!("Invalid GUI specified: '{}'", opt.gui),
    };

    loop {
        nes.step_frame(&mut *gui);
    }
}

#[derive(Debug, StructOpt)]
#[structopt(name = "REZNEZ", about = "The ultra-accurate NES emulator.")]
struct Opt {
    #[structopt(short, long, default_value = "sdl")]
    gui: String,

    #[structopt(name = "ROM", parse(from_os_str))]
    rom_path: PathBuf,
}
