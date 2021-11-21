#![feature(destructuring_assignment)]
#![allow(dead_code)]

mod address;
mod cartridge;
mod cpu;
mod memory;
mod op_code;

use crate::cpu::Cpu;

fn main() {
    let mut cpu = Cpu::startup();
    cpu.step();
}
