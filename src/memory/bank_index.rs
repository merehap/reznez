#[derive(PartialEq, Clone, Copy, Debug)]
pub enum BankIndex {
    IndexFromStart(u16),
    IndexFromEnd(u16),
}

impl BankIndex {
    pub const FIRST: BankIndex = BankIndex::IndexFromStart(0);
    pub const LAST: BankIndex = BankIndex::IndexFromEnd(0);

    pub fn from_u8(value: u8) -> BankIndex {
        BankIndex::IndexFromStart(value.into())
    }

    pub fn to_u16(self, bank_count: u16) -> u16 {
        match self {
            BankIndex::IndexFromStart(index) => index % bank_count,
            BankIndex::IndexFromEnd(index) => {
                assert!(index < bank_count);
                bank_count - index - 1
            }
        }
    }

    pub fn to_usize(self, bank_count: u16) -> usize {
        self.to_u16(bank_count).into()
    }
}

impl From<u8> for BankIndex {
    fn from(value: u8) -> Self {
        BankIndex::IndexFromStart(value.into())
    }
}
