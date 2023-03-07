#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum BankIndex {
    IndexFromStart(u16),
    IndexFromEnd(u16),
    Register(BankIndexRegisterId),
}

impl BankIndex {
    pub const FIRST: BankIndex = BankIndex::IndexFromStart(0);
    pub const SECOND_LAST: BankIndex = BankIndex::IndexFromEnd(1);
    pub const LAST: BankIndex = BankIndex::IndexFromEnd(0);

    pub fn from_u8(value: u8) -> BankIndex {
        BankIndex::IndexFromStart(value.into())
    }

    pub fn to_u16(self, registers: &BankIndexRegisters, bank_count: u16) -> u16 {
        let raw_bank_index = match self {
            BankIndex::IndexFromStart(index) => index,
            BankIndex::IndexFromEnd(index) => {
                assert!(index < bank_count);
                bank_count - index - 1
            }
            // TODO: Get rid of this recursive call.
            BankIndex::Register(id) => registers.get(id)
        };

        raw_bank_index % bank_count
    }

    pub fn to_usize(self, registers: &BankIndexRegisters, bank_count: u16) -> usize {
        self.to_u16(registers, bank_count).into()
    }

    pub fn is_register_backed(self) -> bool {
        matches!(self, BankIndex::Register(_))
    }
}

impl From<u8> for BankIndex {
    fn from(value: u8) -> Self {
        BankIndex::IndexFromStart(value.into())
    }
}

#[derive(Debug)]
pub struct BankIndexRegisters {
    registers: [Option<u16>; 18],
}

impl BankIndexRegisters {
    pub fn new(active_ids: &[BankIndexRegisterId]) -> BankIndexRegisters {
        let mut registers = [None; 18];
        for &id in active_ids {
            registers[id as usize] = Some(0);
        }

        BankIndexRegisters { registers }
    }

    fn get(&self, id: BankIndexRegisterId) -> u16 {
        self.registers[id as usize]
            .unwrap_or_else(|| panic!("Register {id:?} is not configured."))
    }

    pub fn set(&mut self, id: BankIndexRegisterId, index: u16) {
        self.registers[id as usize] = Some(index);
    }

    pub fn merge(&mut self, new_registers: &BankIndexRegisters) {
        for i in 0..self.registers.len() {
            if self.registers[i].is_none() && new_registers.registers[i].is_some() {
                self.registers[i] = new_registers.registers[i];
            }
        }
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
