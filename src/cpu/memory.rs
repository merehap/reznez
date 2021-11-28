use std::ops::{Index, IndexMut};

use crate::cpu::address::Address;

pub struct Memory {
    pub stack_pointer: u8,
    memory: [u8; 0x10000],
}

impl Memory {
    pub fn startup() -> Memory {
        Memory {
            stack_pointer: 0xFD,
            memory: [0; 0x10000],
        }
    }

    pub fn push(&mut self, value: u8) {
        if self.stack_pointer == 0 {
            panic!("Cannot push to a full stack.");
        }

        self.memory[self.stack_pointer as usize + 0x100] = value;
        self.stack_pointer -= 1;
    }

    pub fn push_address(&mut self, address: Address) {
        let (low, high) = address.to_low_high();
        self.push(high);
        self.push(low);
    }

    pub fn pop(&mut self) -> u8 {
        if self.stack_pointer == 0xFF {
            panic!("Cannot pop from an empty stack.");
        }

        self.stack_pointer += 1;
        self.memory[self.stack_pointer as usize + 0x100]
    }

    pub fn pop_address(&mut self) -> Address {
        let low = self.pop();
        let high = self.pop();
        Address::from_low_high(low, high)
    }

    pub fn address_from_vector(&self, mut vector: Address) -> Address {
        Address::from_low_high(self[vector], self[vector.inc()])
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
