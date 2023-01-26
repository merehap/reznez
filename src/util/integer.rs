#[derive(Default)]
pub struct U3(u8);

impl From<u8> for U3 {
    fn from(value: u8) -> Self {
        U3(value & 0b0000_0111)
    }
}

#[derive(Default)]
pub struct U4(u8);

impl From<u8> for U4 {
    fn from(value: u8) -> Self {
        U4(value & 0b0000_1111)
    }
}

#[derive(PartialEq, Eq, Default)]
pub struct U5(u8);

impl U5 {
    pub const ZERO: U5 = U5(0);

    fn decrement(&mut self) {
        assert!(self.0 != 0);
        self.0 -= 1;
    }
}

impl From<u8> for U5 {
    fn from(value: u8) -> Self {
        U5(value & 0b0001_1111)
    }
}

#[derive(Default)]
pub struct U7(u8);

impl From<u8> for U7 {
    fn from(value: u8) -> Self {
        U7(value & 0b0111_1111)
    }
}
