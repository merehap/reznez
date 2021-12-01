use std::ops::{Index, IndexMut};

use crate::ppu::sprite::Sprite;

pub struct Oam([u8; 256]);

impl Oam {
    pub fn new() -> Oam {
        Oam([0; 256])
    }

    pub fn sprites(&self) -> [Sprite; 64] {
        // Ugh, should just give up and use transmute.
        self.0.array_chunks::<4>()
            .map(|chunk| Sprite::from_u32(u32::from_be_bytes(*chunk)))
            .collect::<Vec<_>>()
            .try_into()
            .unwrap()
    }
}

impl Index<u8> for Oam {
    type Output = u8;

    fn index(&self, index: u8) -> &u8 {
        &self.0[index as usize]
    }
}

impl IndexMut<u8> for Oam {
    fn index_mut(&mut self, index: u8) -> &mut u8 {
        &mut self.0[index as usize]
    }
}
