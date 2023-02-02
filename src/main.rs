#![feature(array_chunks)]
#![feature(slice_as_chunks)]
#![feature(const_option)]
#![feature(if_let_guard)]
#![feature(let_chains)]
#![feature(type_ascription)]
#![feature(const_option_ext)]
#![allow(clippy::module_inception)]
#![allow(clippy::new_without_default)]

mod apu;
mod analysis;
mod cartridge;
mod config;
mod controller;
mod cpu;
mod gui;
mod memory;
pub mod nes;
mod ppu;
mod util;

use structopt::StructOpt;

use crate::config::{Config, Opt};
use crate::nes::Nes;
use crate::util::logger;
use crate::util::logger::Logger;

fn main() {
    let opt = Opt::from_args();
    logger::init(Logger {
        log_cpu_operations: opt.log_cpu_operations,
        log_cpu_steps: opt.log_cpu_steps,
    }).unwrap();

    if opt.analysis {
        analysis::cartridge_db::analyze(&opt.rom_path);
    } else {
        let config = Config::new(&opt);
        let mut gui = Config::gui(&opt);
        let nes = Nes::new(&config);

        gui.run(nes, config);
    }
}
