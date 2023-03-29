#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct BankIndex(u16);

impl BankIndex {
    pub const FIRST: BankIndex = BankIndex(0);
    pub const SECOND_LAST: BankIndex = BankIndex(0xFFFE);
    pub const LAST: BankIndex = BankIndex(0xFFFF);

    pub const fn from_u8(value: u8) -> BankIndex {
        BankIndex(value as u16)
    }

    pub const fn from_u16(value: u16) -> BankIndex {
        BankIndex(value)
    }

    pub fn to_u16(self, bank_count: u16) -> u16 {
        self.0 % bank_count
    }

    pub fn to_usize(self, bank_count: u16) -> usize {
        self.to_u16(bank_count).into()
    }
}

impl From<u8> for BankIndex {
    fn from(value: u8) -> Self {
        BankIndex(value.into())
    }
}

#[derive(Debug)]
pub struct BankIndexRegisters {
    registers: [BankIndex; 18],
}

impl BankIndexRegisters {
    pub fn new() -> BankIndexRegisters {
        BankIndexRegisters { registers: [BankIndex::FIRST; 18] }
    }

    pub fn get(&self, id: BankIndexRegisterId) -> BankIndex {
        self.registers[id as usize]
    }

    pub fn set(&mut self, id: BankIndexRegisterId, bank_index: BankIndex) {
        self.registers[id as usize] = bank_index;
    }

    pub fn update(&mut self, id: BankIndexRegisterId, updater: &dyn Fn(u16) -> u16) {
        let value = self.registers[id as usize].0;
        self.registers[id as usize] = BankIndex(updater(value));
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum BankIndexRegisterId {
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
