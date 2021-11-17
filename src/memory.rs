use std::ops::{Index, IndexMut};

use crate::address::Address;

pub struct Memory {
    memory: [u8; 0x10000],
}

impl Memory {
    pub fn startup() -> Memory {
        Memory {memory: [0; 0x10000]}
    }
}

impl Index<Address> for Memory {
    type Output = u8;

    fn index(&self, address: Address) -> &Self::Output {
        &self.memory[address.to_raw() as usize]
    }
}

impl IndexMut<Address> for Memory {
    fn index_mut(&mut self, address: Address) -> &mut Self::Output {
        &mut self.memory[address.to_raw() as usize]
    }
}

