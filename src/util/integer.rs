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

#[derive(Default)]
pub struct U11(u16);

impl U11 {
    pub fn set_low_byte(&mut self, value: u8) {
        self.0 &=          0b0000_0111_0000_0000;
        self.0 |= u16::from(value) & 0b0000_0000_1111_1111;
    }

    pub fn set_high_bits(&mut self, value: u8) {
        self.0 &=                           0b0000_0000_1111_1111;
        self.0 |= (u16::from(value) << 8) & 0b0000_0111_0000_0000;
    }
}
