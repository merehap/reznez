#![feature(array_chunks)]
#![feature(slice_as_chunks)]
#![feature(const_option)]
#![feature(if_let_guard)]
#![feature(type_ascription)]
#![allow(clippy::module_inception)]
#![allow(clippy::new_without_default)]

mod analysis;
mod cartridge;
mod config;
mod controller;
mod cpu;
mod gui;
mod ppu;
mod memory;
pub mod nes;
mod util;

use structopt::StructOpt;

use crate::config::{Config, Opt};
use crate::nes::Nes;
use crate::util::logger;
use crate::util::logger::Logger;

fn main() {
    let opt = Opt::from_args();
    logger::init(Logger {log_cpu: opt.log_cpu}).unwrap();
    if opt.analysis {
        analysis::cartridge_db::analyze(&opt.rom_path);
    } else {
        let config = Config::new(&opt);
        let mut gui = Config::gui(&opt);
        let nes = Nes::new(&config);

        gui.run(nes, config);
    }
}
