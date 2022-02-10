use std::ops::{Index, IndexMut};

use crate::memory::cpu::cpu_address::CpuAddress;
use crate::memory::cpu::stack::Stack;

pub const NMI_VECTOR: CpuAddress = CpuAddress::new(0xFFFA);
pub const RESET_VECTOR: CpuAddress = CpuAddress::new(0xFFFC);
pub const IRQ_VECTOR: CpuAddress = CpuAddress::new(0xFFFE);

const RAM_SIZE: usize = 0x2000;
const STACK_START: usize = 0x100;
const STACK_END: usize = 0x1FF;

const STARTUP_STACK_POINTER: u8 = 0xFD;

pub struct CpuInternalRam {
    pub stack_pointer: u8,
    memory: Box<[u8; RAM_SIZE]>,
}

impl CpuInternalRam {
    pub fn new() -> CpuInternalRam {
        CpuInternalRam {
            stack_pointer: STARTUP_STACK_POINTER,
            memory: Box::new([0; RAM_SIZE]),
        }
    }

    pub fn stack(&mut self) -> Stack {
        Stack::new(
            (&mut self.memory[STACK_START..=STACK_END]).try_into().unwrap(),
            &mut self.stack_pointer,
        )
    }
}

impl Index<usize> for CpuInternalRam {
    type Output = u8;

    fn index(&self, idx: usize) -> &u8 {
        &self.memory[idx]
    }
}

impl IndexMut<usize> for CpuInternalRam {
    fn index_mut(&mut self, idx: usize) -> &mut u8 {
        &mut self.memory[idx]
    }
}
