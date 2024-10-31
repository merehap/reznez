use crate::memory::bank::bank::RamStatusRegisterId;

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct BankIndex(u16);

impl BankIndex {
    pub const FIRST: BankIndex = BankIndex(0);
    pub const SECOND: BankIndex = BankIndex(1);
    // Rely on later bit masking to reduce these indexes to within the valid range.
    pub const THIRD_LAST: BankIndex = BankIndex(0xFFFD);
    pub const SECOND_LAST: BankIndex = BankIndex(0xFFFE);
    pub const LAST: BankIndex = BankIndex(0xFFFF);

    pub const fn from_u8(value: u8) -> BankIndex {
        BankIndex(value as u16)
    }

    pub const fn from_u16(value: u16) -> BankIndex {
        BankIndex(value)
    }

    pub const fn from_i16(value: i16) -> BankIndex {
        if value >= 0 {
            BankIndex(value as u16)
        } else {
            // Negative values wrap around to count down from u16::MAX.
            BankIndex(u16::MAX - value.unsigned_abs() + 1)
        }
    }

    pub fn to_u16(self, bank_count: u16) -> u16 {
        self.0 % bank_count
    }

    pub fn to_u32(self, bank_count: u16) -> u32 {
        self.to_u16(bank_count).into()
    }
}

impl From<u8> for BankIndex {
    fn from(value: u8) -> Self {
        BankIndex(value.into())
    }
}

#[derive(Debug)]
pub struct BankRegisters {
    registers: [BankIndex; 18],
    meta_registers: [BankRegisterId; 2],
    ram_statuses: [RamStatus; 2],
}

impl BankRegisters {
    pub fn new() -> BankRegisters {
        BankRegisters {
            registers: [BankIndex::FIRST; 18],
            // Meta registers are only used for CHR currently.
            meta_registers: [BankRegisterId::C0, BankRegisterId::C0],
            ram_statuses: [RamStatus::ReadWrite, RamStatus::ReadWrite],
        }
    }

    pub fn get(&self, id: BankRegisterId) -> BankIndex {
        self.registers[id as usize]
    }

    pub fn set(&mut self, id: BankRegisterId, bank_index: BankIndex) {
        self.registers[id as usize] = bank_index;
    }

    pub fn set_bits(&mut self, id: BankRegisterId, new_value: u16, mask: u16) {
        let value = self.registers[id as usize].0;
        let updated_value = (value & !mask) | (new_value & mask);
        self.registers[id as usize] = BankIndex(updated_value);
    }

    pub fn update(&mut self, id: BankRegisterId, updater: &dyn Fn(u16) -> u16) {
        let value = self.registers[id as usize].0;
        self.registers[id as usize] = BankIndex(updater(value));
    }

    pub fn get_from_meta(&self, id: MetaRegisterId) -> BankIndex {
        self.get(self.meta_registers[id as usize])
    }

    pub fn set_meta(&mut self, id: MetaRegisterId, value: BankRegisterId) {
        self.meta_registers[id as usize] = value;
    }

    pub fn ram_status(&self, id: RamStatusRegisterId) -> RamStatus {
        self.ram_statuses[id as usize]
    }

    pub fn set_ram_status(&mut self, id: RamStatusRegisterId, status: RamStatus) {
        self.ram_statuses[id as usize] = status;
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum BankRegisterId {
    C0,
    C1,
    C2,
    C3,
    C4,
    C5,
    C6,
    C7,
    C8,
    C9,
    C10,
    C11,
    C12,

    P0,
    P1,
    P2,
    P3,
    P4,
}

impl BankRegisterId {
    pub fn chr_id(id: u16) -> BankRegisterId {
        use BankRegisterId::*;
        match id {
            0 => C0,
            1 => C1,
            2 => C2,
            3 => C3,
            4 => C4,
            5 => C5,
            6 => C6,
            7 => C7,
            8 => C8,
            9 => C9,
            10 => C10,
            11 => C11,
            12 => C12,
            _ => panic!("Bad CHR ID: {id}"),
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum MetaRegisterId {
    M0,
    M1,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum RamStatus {
    Disabled,
    ReadOnlyZeros,
    ReadOnly,
    ReadWrite,
}
