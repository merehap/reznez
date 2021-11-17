mod address;
mod cpu;
mod memory;
mod op_code;

use crate::cpu::Cpu;

fn main() {
    let _cpu = Cpu::startup();
}
