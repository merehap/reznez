// Clippy bug.
#![allow(clippy::needless_borrow)]

use crate::ppu::register::ppu_registers::PpuRegisters;
use crate::util::unit::KIBIBYTE;

const CIRAM_SIZE: usize = 2 * KIBIBYTE as usize;
const CHUNK_SIZE: usize = KIBIBYTE as usize;

// Console-internal name table memory.
pub struct Ciram(Box<[u8; CIRAM_SIZE]>);

impl Ciram {
    pub fn new() -> Ciram {
        Ciram(Box::new([0; CIRAM_SIZE]))
    }

    pub fn side(&self, side: CiramSide) -> &[u8; CHUNK_SIZE] {
        let start_index = side as usize;
        self.0[start_index..start_index + CHUNK_SIZE]
            .try_into()
            .unwrap()
    }

    pub fn write(&mut self, regs: &PpuRegisters, side: CiramSide, index: u16, value: u8) {
        if !regs.reset_recently() {
            self.0[usize::from(side as u16 + index)] = value;
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum CiramSide {
    Left = 0,
    Right = CHUNK_SIZE as isize,
}
