use std::ops::{Index, IndexMut};

const PALETTE_RAM_SIZE: usize = 0x20;

pub struct PaletteRam([u8; PALETTE_RAM_SIZE]);

impl PaletteRam {
    pub fn new() -> PaletteRam {
        PaletteRam([0; PALETTE_RAM_SIZE])
    }

    pub fn to_slice(&self) -> &[u8; PALETTE_RAM_SIZE] {
        &self.0
    }
}

impl Index<usize> for PaletteRam {
    type Output = u8;

    fn index(&self, idx: usize) -> &u8 {
        &self.0[idx]
    }
}

impl IndexMut<usize> for PaletteRam {
    fn index_mut(&mut self, idx: usize) -> &mut u8 {
        &mut self.0[idx]
    }
}
