#![feature(destructuring_assignment)]
#![allow(dead_code)]

mod cartridge;
mod cpu;
mod mapper;
mod nes;
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
    let mut nes = Nes::startup(ines);

    for _ in 0..10 {
        nes.step();
    }
}
