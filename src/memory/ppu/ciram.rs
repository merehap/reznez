// Clippy bug.
#![allow(clippy::needless_borrow)]

use std::ops::{Index, IndexMut};

use crate::util::unit::KIBIBYTE;

const VRAM_SIZE: usize = 2 * KIBIBYTE as usize;
const CHUNK_SIZE: usize = KIBIBYTE as usize;

// Console-internal name table memory.
pub struct Ciram(Box<[u8; VRAM_SIZE]>);

impl Ciram {
    pub fn new() -> Ciram {
        Ciram(Box::new([0; VRAM_SIZE]))
    }

    pub fn side(&self, side: CiramSide) -> &[u8; CHUNK_SIZE] {
        let start_index = side as usize;
        self.0[start_index..start_index + CHUNK_SIZE]
            .try_into()
            .unwrap()
    }

    pub fn side_mut(&mut self, side: CiramSide) -> &mut [u8; CHUNK_SIZE] {
        let start_index = side as usize;
        (&mut self.0[start_index..start_index + CHUNK_SIZE])
            .try_into()
            .unwrap()
    }
}

impl Index<u16> for Ciram {
    type Output = u8;

    fn index(&self, idx: u16) -> &u8 {
        &self.0[idx as usize]
    }
}

impl IndexMut<u16> for Ciram {
    fn index_mut(&mut self, idx: u16) -> &mut u8 {
        &mut self.0[idx as usize]
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum CiramSide {
    Left = 0,
    Right = CHUNK_SIZE as isize,
}
