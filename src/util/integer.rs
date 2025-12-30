#[derive(PartialEq, Eq, Clone, Copy, Default)]
pub struct U7(u8);

impl U7 {
    pub const ZERO: U7 = U7(0);

    pub fn decrement_towards_zero(&mut self) {
        if self.0 > 0 {
            self.0 -= 1;
        }
    }
}

impl From<u8> for U7 {
    fn from(value: u8) -> Self {
        U7(value & 0b0111_1111)
    }
}
