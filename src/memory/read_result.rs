// TODO: Rename to PeekResult.
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct ReadResult {
    value: u8,
    mask: u8,
    bus_update_needed: bool,
}

impl ReadResult {
    pub const OPEN_BUS: Self = Self { value: 0, mask: 0b0000_0000, bus_update_needed: true };

    pub fn full(value: u8) -> Self {
        Self { value, mask: 0b1111_1111, bus_update_needed: true }
    }

    pub fn partial(value: u8, mask: u8) -> Self {
        Self { value, mask, bus_update_needed: true }
    }

    pub fn no_bus_update(value: u8, mask: u8) -> Self {
        Self { value, mask, bus_update_needed: false }
    }

    pub fn resolve(self, data_bus_value: u8) -> (u8, bool) {
        let value = (self.value & self.mask) | (data_bus_value & !self.mask);
        (value, self.bus_update_needed)
    }

    // Bus conflicts occur when a register exists at the same address as ROM, for boards that don't
    // prevent bus conflicts. This results in the register value and the ROM value being ANDed when
    // a register write occurs.
    // If self.mask is all zeroes (a.k.a. open bus), then the register is written unmodified.
    // If self.mask is all ones, then each register bit conflicts with ROM (and will be ANDed).
    pub fn bus_conflict(self, register_value: u8) -> u8 {
        assert!(self.mask == 0b0000_0000 || self.mask == 0b1111_1111,
            "ReadResult for a potential bus conflict should not be partial open bus.");
        (self.value & register_value & self.mask) | (register_value & !self.mask)
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
