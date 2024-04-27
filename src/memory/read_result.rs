#[derive(Clone, Copy)]
pub struct ReadResult {
    value: u8,
    mask: u8,
}

impl ReadResult {
    pub const OPEN_BUS: Self = Self { value: 0, mask: 0b0000_0000 };

    pub fn full(value: u8) -> Self {
        Self { value, mask: 0b1111_1111 }
    }

    pub fn partial_open_bus(value: u8, mask: u8) -> Self {
        Self { value, mask }
    }

    pub fn resolve(self, data_bus_value: u8) -> u8 {
        (self.value & self.mask) | (data_bus_value & !self.mask)
    }

    pub fn unwrap(self) -> u8 {
        assert_eq!(self.mask, 0b1111_1111);
        self.value
    }

    pub fn unwrap_or(self, value: u8) -> u8 {
        match self.mask {
            0b1111_1111 => self.value,
            _ => value,
        }
    }

    pub fn expect(self, message: &str) -> u8 {
        assert_eq!(self.mask, 0b1111_1111, "{message}");
        self.value
    }
}
