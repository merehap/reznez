use std::collections::BTreeSet;

use crate::cpu::address::Address;
use crate::cpu::port_access::{PortAccess, AccessMode};

const NMI_VECTOR: Address = Address::new(0xFFFA);
const RESET_VECTOR: Address = Address::new(0xFFFC);
const IRQ_VECTOR: Address = Address::new(0xFFFE);

pub struct Memory {
    pub stack_pointer: u8,
    memory: [u8; 0x10000],
    read_ports: BTreeSet<Address>,
    write_ports: BTreeSet<Address>,
    latch: Option<PortAccess>,
}

impl Memory {
    pub fn new(read_ports: BTreeSet<Address>, write_ports: BTreeSet<Address>) -> Memory {
        Memory {
            stack_pointer: 0xFD,
            memory: [0; 0x10000],
            read_ports,
            write_ports,
            latch: None,
        }
    }

    pub fn read(&mut self, address: Address) -> u8 {
        let value = self.memory[address.to_raw() as usize];
        if self.read_ports.contains(&address) {
            self.latch = Some(PortAccess {
                address,
                value,
                access_mode: AccessMode::Read,
            });
        }

        value
    }

    pub fn write(&mut self, address: Address, value: u8) {
        if self.write_ports.contains(&address) {
            self.latch = Some(PortAccess {
                address,
                value,
                access_mode: AccessMode::Write,
            });
        }

        self.memory[address.to_raw() as usize] = value;
    }

    pub fn bus_access(&self, address: Address) -> &u8 {
        &self.memory[address.to_raw() as usize]
    }

    pub fn bus_access_mut(&mut self, address: Address) -> &mut u8 {
        &mut self.memory[address.to_raw() as usize]
    }

    pub fn push_to_stack(&mut self, value: u8) {
        if self.stack_pointer == 0x00 {
            println!("Pushing to full stack.");
        }

        self.memory[self.stack_pointer as usize + 0x100] = value;
        self.stack_pointer = self.stack_pointer.wrapping_sub(1);
    }

    pub fn push_address_to_stack(&mut self, address: Address) {
        let (low, high) = address.to_low_high();
        self.push_to_stack(high);
        self.push_to_stack(low);
    }

    pub fn pop_from_stack(&mut self) -> u8 {
        if self.stack_pointer == 0xFF {
            println!("Popping from empty stack.");
        }

        self.stack_pointer = self.stack_pointer.wrapping_add(1);
        self.memory[self.stack_pointer as usize + 0x100]
    }

    pub fn pop_address_from_stack(&mut self) -> Address {
        let low = self.pop_from_stack();
        let high = self.pop_from_stack();
        Address::from_low_high(low, high)
    }

    pub fn latch(&self) -> Option<PortAccess> {
        self.latch
    }

    pub fn reset_latch(&mut self) {
        self.latch = None;
    }

    pub fn stack(&self) -> &[u8] {
        &self.memory[self.stack_pointer as usize + 0x101..0x200]
    }

    pub fn slice(&self, start_address: Address, length: u16) -> &[u8] {
        let start_address = start_address.to_raw() as usize;
        &self.memory[start_address..start_address + length as usize]
    }

    pub fn nmi_vector(&self) -> Address {
        self.address_from_vector(NMI_VECTOR)
    }

    pub fn reset_vector(&self) -> Address {
        self.address_from_vector(RESET_VECTOR)
    }

    pub fn irq_vector(&self) -> Address {
        self.address_from_vector(IRQ_VECTOR)
    }

    fn address_from_vector(&self, mut vector: Address) -> Address {
        Address::from_low_high(
            self.memory[vector.to_raw() as usize],
            self.memory[vector.inc().to_raw() as usize],
            )
    }

}
