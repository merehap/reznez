use log::info;

use crate::memory::cpu::cpu_address::CpuAddress;

pub struct Stack<'a> {
    raw: &'a mut [u8; 0x100],
    pointer: &'a mut u8,
}

impl<'a> Stack<'a> {
    pub fn new(raw: &'a mut [u8; 0x100], pointer: &'a mut u8) -> Stack<'a> {
        Stack { raw, pointer }
    }

    pub fn set_at_pointer(&mut self, value: u8) {
        self.raw[*self.pointer as usize] = value;
    }

    pub fn push(&mut self, value: u8) {
        if *self.pointer == 0x00 {
            info!("Pushing to full stack. Wrapping around.");
        }

        self.raw[*self.pointer as usize] = value;
        *self.pointer = self.pointer.wrapping_sub(1);
    }

    pub fn push_address(&mut self, address: CpuAddress) {
        let (low, high) = address.to_low_high();
        self.push(high);
        self.push(low);
    }

    pub fn peek(&self) -> u8 {
        self.raw[*self.pointer as usize]
    }

    pub fn pop(&mut self) -> u8 {
        if *self.pointer == 0xFF {
            info!("Popping from empty stack. Wrapping around.");
        }

        *self.pointer = self.pointer.wrapping_add(1);
        self.raw[*self.pointer as usize]
    }

    pub fn pop_address(&mut self) -> CpuAddress {
        let low = self.pop();
        let high = self.pop();
        CpuAddress::from_low_high(low, high)
    }

    pub fn increment_stack_pointer(&mut self) {
        *self.pointer = self.pointer.wrapping_add(1);
    }

    pub fn decrement_stack_pointer(&mut self) {
        *self.pointer = self.pointer.wrapping_sub(1);
    }
}
