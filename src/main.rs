#![feature(array_chunks)]
#![feature(const_option)]
#![feature(destructuring_assignment)]
#![allow(dead_code)]
#![allow(clippy::module_inception)]

mod cartridge;
mod cpu;
mod gui;
mod ppu;
mod mapper;
pub mod nes;
mod util;

use std::io::Read;
use std::fs::File;

use crate::cartridge::INes;
use crate::nes::Nes;

fn main() {
    let mut rom = Vec::new();
    File::open("roms/nestest.nes")
        .unwrap()
        .read_to_end(&mut rom)
        .unwrap();

    let ines = INes::load(&rom).unwrap();
    let nes = Nes::startup(ines);

    gui::gui(nes).unwrap();
}
