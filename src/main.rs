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

use structopt::StructOpt;

use crate::config::{Config, Opt};
use crate::nes::Nes;
use crate::util::logger;
use crate::util::logger::Logger;

fn main() {
    let opt = Opt::from_args();
    logger::init(Logger {log_cpu: opt.log_cpu}).unwrap();
    let config = Config::new(&opt);
    let mut gui = Config::gui(&opt);
    let mut nes = Nes::new(config);

    loop {
        nes.step_frame(&mut *gui);
    }
}
