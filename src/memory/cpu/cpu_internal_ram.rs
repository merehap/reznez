// Clippy bug.
#![allow(clippy::needless_borrow)]

use std::ops::{Index, IndexMut};

const RAM_SIZE: usize = 0x2000;

pub struct CpuInternalRam(Box<[u8; RAM_SIZE]>);

impl CpuInternalRam {
    pub fn new() -> CpuInternalRam {
        CpuInternalRam(Box::new([0; RAM_SIZE]))
    }
}

impl Index<usize> for CpuInternalRam {
    type Output = u8;

    fn index(&self, idx: usize) -> &u8 {
        &self.0[idx]
    }
}

impl IndexMut<usize> for CpuInternalRam {
    fn index_mut(&mut self, idx: usize) -> &mut u8 {
        &mut self.0[idx]
    }
}
