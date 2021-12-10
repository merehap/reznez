use std::ops::{Index, IndexMut};

use crate::ppu::address::Address;
use crate::ppu::name_table::NameTable;
use crate::ppu::name_table_number::NameTableNumber;
use crate::ppu::pattern_table::PatternTable;
use crate::ppu::palette::palette_table::PaletteTable;

const MEMORY_SIZE: usize = 0x4000;

const PATTERN_TABLE_START: Address = Address::from_u16(0);
const PATTERN_TABLE_SIZE: u16 = 0x2000;

const NAME_TABLE_START: u16 = 0x2000;
const NAME_TABLE_SIZE: u16 = 0x400;
const NAME_TABLE_INDEXES: [Address; 4] =
    [
        Address::from_u16(NAME_TABLE_START + 0 * NAME_TABLE_SIZE),
        Address::from_u16(NAME_TABLE_START + 1 * NAME_TABLE_SIZE),
        Address::from_u16(NAME_TABLE_START + 2 * NAME_TABLE_SIZE),
        Address::from_u16(NAME_TABLE_START + 3 * NAME_TABLE_SIZE),
    ];

const PALETTE_TABLE_START: Address = Address::from_u16(0x3F00);
const PALETTE_TABLE_SIZE: u16 = 0x20;

pub struct Memory {
    memory: [u8; MEMORY_SIZE],
}

impl Memory {
    pub fn new() -> Memory {
        Memory {
            memory: [0; MEMORY_SIZE],
        }
    }

    pub fn pattern_table(&self) -> PatternTable {
        let raw = self.slice(PATTERN_TABLE_START, PATTERN_TABLE_SIZE);
        PatternTable::new(raw.try_into().unwrap())
    }

    pub fn name_table(&self, number: NameTableNumber) -> NameTable {
        let index = NAME_TABLE_INDEXES[number as usize];
        let raw = self.slice(index, NAME_TABLE_SIZE);
        NameTable::new(raw.try_into().unwrap())
    }

    pub fn palette_table(&self) -> PaletteTable {
        let raw = self.slice(PALETTE_TABLE_START, PALETTE_TABLE_SIZE);
        PaletteTable::new(raw.try_into().unwrap())
    }

    fn slice(&self, start_address: Address, length: u16) -> &[u8] {
        let start_address = start_address.to_u16() as usize;
        &self.memory[start_address..start_address + length as usize]
    }
}

impl Index<Address> for Memory {
    type Output = u8;

    fn index(&self, address: Address) -> &Self::Output {
        &self.memory[address.to_u16() as usize]
    }
}

impl IndexMut<Address> for Memory {
    fn index_mut(&mut self, address: Address) -> &mut Self::Output {
        &mut self.memory[address.to_u16() as usize]
    }
}
