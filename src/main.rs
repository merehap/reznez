#![feature(destructuring_assignment)]

mod address;
mod cpu;
mod memory;
mod op_code;

use crate::cpu::Cpu;

fn main() {
    let mut cpu = Cpu::startup();
    cpu.step();
}
