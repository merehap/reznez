#[derive(Clone, Copy, Default)]
pub struct U3(u8);

impl From<u8> for U3 {
    fn from(value: u8) -> Self {
        U3(value & 0b0000_0111)
    }
}

#[derive(Clone, Copy, Default)]
pub struct U4(u8);

impl U4 {
    pub fn to_u8(self) -> u8 {
        self.0
    }
}

impl From<u8> for U4 {
    fn from(value: u8) -> Self {
        U4(value & 0b0000_1111)
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Default)]
pub struct U7(u8);

impl U7 {
    pub const ZERO: U7 = U7(0);

    pub fn decrement_towards_zero(&mut self) {
        if self.0 > 0 {
            self.0 -= 1;
        }
    }

    pub fn to_u8(self) -> u8 {
        self.0
    }
}

impl From<u8> for U7 {
    fn from(value: u8) -> Self {
        U7(value & 0b0111_1111)
    }
}
