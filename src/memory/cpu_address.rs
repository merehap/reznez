use std::fmt;
use std::str::FromStr;

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Debug)]
pub struct CpuAddress(u16);

impl CpuAddress {
    pub const fn new(value: u16) -> CpuAddress {
        CpuAddress(value)
    }

    pub fn from_low_high(low: u8, high: u8) -> CpuAddress {
        CpuAddress::new(((u16::from(high)) << 8) + (u16::from(low)))
    }

    pub fn zero_page(low: u8) -> CpuAddress {
        CpuAddress::new(u16::from(low))
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

    pub fn advance(self, value: u8) -> CpuAddress {
        CpuAddress::new(self.0.wrapping_add(u16::from(value)))
    }

    pub fn offset(self, value: i8) -> CpuAddress {
        CpuAddress::new((i32::from(self.0)).wrapping_add(i32::from(value)) as u16)
    }

    pub fn inc(&mut self) -> CpuAddress {
        self.0 = self.0.wrapping_add(1);
        *self
    }

    pub fn page(self) -> u8 {
        (self.0 >> 8) as u8
    }
}

impl fmt::Display for CpuAddress {
    fn fmt<'a>(&self, f: &mut std::fmt::Formatter<'a>) -> fmt::Result {
        write!(f, "${:04X}", self.0)
    }
}

impl FromStr for CpuAddress {
    type Err = String;

    fn from_str(value: &str) -> Result<CpuAddress, String> {
        let raw = u16::from_str(value)
            .map_err(|err| err.to_string())?;
        Ok(CpuAddress(raw))
    }
}
