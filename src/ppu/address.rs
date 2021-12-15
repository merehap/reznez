use std::fmt;

const MAX_ADDRESS: u16 = 0x3FFF;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub struct Address(u16);

impl Address {
    pub const fn from_u16(mut value: u16) -> Address {
        if value > MAX_ADDRESS {
            // https://wiki.nesdev.org/w/index.php?title=PPU_registers#PPUADDR
            value &= MAX_ADDRESS;
        }

        // Map the name table mirrors.
        if value >= 0x3000 && value < 0x3F00 {
            value -= 0x1000;
        }

        // Map the palette RAM index mirrors.
        if value >= 0x3F20 {
            value = 0x3F00 + value % 0x20;
        }

        Address(value)
    }

    pub const fn advance(&self, value: u8) -> Address {
        Address::from_u16(self.0 + value as u16)
    }

    pub fn inc(&mut self) -> Address {
        self.0 = self.0.wrapping_add(1);
        *self
    }

    pub fn to_u16(&self) -> u16 {
        self.0
    }
}

impl fmt::Display for Address {
    fn fmt<'a>(&self, f: &mut std::fmt::Formatter<'a>) -> fmt::Result {
        write!(f, "${:04X}", self.0)
    }
}
