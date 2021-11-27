#![feature(destructuring_assignment)]
#![allow(dead_code)]

mod address;
mod cartridge;
mod cpu;
mod mapper;
mod memory;
mod op_code;

use std::io::Read;
use std::fs::File;

use crate::cartridge::INes;
use crate::cpu::Cpu;

fn main() {
    let mut rom = Vec::new();
    File::open("roms/thwaite.nes")
        .unwrap()
        .read_to_end(&mut rom)
        .unwrap();

    let ines = INes::load(&rom).unwrap();
    let mut cpu = Cpu::startup(ines);

    for _ in 0..10000 {
        cpu.step();
    }
}
