use std::fmt;
use std::str::FromStr;

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Debug)]
#[repr(transparent)]
pub struct Address(u16);

impl Address {
    pub const fn new(mut value: u16) -> Address {
        if value < 0x2000 {
            // Map RAM mirrors to the true RAM range.
            value %= 0x2000;
        } else if value >= 0x2000 && value < 0x4000 {
            // Map PPU register mirrors to the true PPU register range.
            value = (value % 0x8) + 0x2000;
        }

        Address(value)
    }

    pub fn from_low_high(low: u8, high: u8) -> Address {
        Address::new(((u16::from(high)) << 8) + (u16::from(low)))
    }

    pub fn zero_page(low: u8) -> Address {
        Address::new(u16::from(low))
    }

    pub fn to_raw(self) -> u16 {
        self.0
    }

    pub fn to_usize(self) -> usize {
        usize::from(self.0)
    }

    pub fn to_low_high(self) -> (u8, u8) {
        (self.0 as u8, (self.0 >> 8) as u8)
    }

    pub fn advance(self, value: u8) -> Address {
        Address::new(self.0.wrapping_add(u16::from(value)))
    }

    pub fn offset(self, value: i8) -> Address {
        Address::new((i32::from(self.0)).wrapping_add(i32::from(value)) as u16)
    }

    pub fn inc(&mut self) -> Address {
        self.0 = self.0.wrapping_add(1);
        *self
    }

    pub fn page(self) -> u8 {
        (self.0 >> 8) as u8
    }

    pub fn get_type(self) -> AddressType {
        use AddressType::*;
        match self.0 {
            0x0000..=0x07FF => InternalRAM,
            // Internal RAM mirrors omitted here.
            0x2000..=0x2007 => PpuRegister,
            // PPU register mirrors omitted here.
            0x4000..=0x4017 => ApuRegister,
            0x4018..=0x401F => DisabledApuRegister,
            0x4020..=0xFFF9 => Cartridge,
            0xFFFA..=0xFFFF => InterruptVector,
            _ => unreachable!(
                "Value '{}' should not be possible for an Address.",
                self.0,
                ),
        }
    }
}

impl fmt::Display for Address {
    fn fmt<'a>(&self, f: &mut std::fmt::Formatter<'a>) -> fmt::Result {
        write!(f, "${:04X}", self.0)
    }
}

impl FromStr for Address {
    type Err = String;

    fn from_str(value: &str) -> Result<Address, String> {
        let raw = u16::from_str(value)
            .map_err(|err| err.to_string())?;
        Ok(Address(raw))
    }
}

pub enum AddressType {
    InternalRAM,
    PpuRegister,
    ApuRegister,
    DisabledApuRegister,
    Cartridge,
    InterruptVector,
}
