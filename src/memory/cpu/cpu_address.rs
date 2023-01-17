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

    pub fn low_byte(self) -> u8 {
        u8::try_from(self.0 & 0x00FF).unwrap()
    }

    pub fn high_byte(self) -> u8 {
        u8::try_from(self.0 >> 8).unwrap()
    }

    pub fn advance(self, value: u8) -> CpuAddress {
        CpuAddress::new(self.0.wrapping_add(u16::from(value)))
    }

    pub fn offset(self, value: i8) -> CpuAddress {
        CpuAddress::new((i32::from(self.0)).wrapping_add(i32::from(value)) as u16)
    }

    pub fn offset_low(&mut self, value: u8) -> bool {
        let (low, high) = self.to_low_high();
        let (low, carry) = low.overflowing_add(value);
        *self = CpuAddress::from_low_high(low, high);
        carry
    }

    pub fn offset_high(&mut self, value: i8) {
        let (low, high) = self.to_low_high();
        let high = high.wrapping_add_signed(value);
        *self = CpuAddress::from_low_high(low, high);
    }

    pub fn inc(&mut self) -> CpuAddress {
        self.0 = self.0.wrapping_add(1);
        *self
    }

    pub fn page(self) -> u8 {
        (self.0 >> 8) as u8
    }

    pub fn index_within_page(self) -> u8 {
        self.0 as u8
    }

    pub fn is_end_of_page(self) -> bool {
        self.index_within_page() == 0xFF
    }
}

impl fmt::Display for CpuAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "${:04X}", self.0)
    }
}

impl FromStr for CpuAddress {
    type Err = String;

    fn from_str(value: &str) -> Result<CpuAddress, String> {
        let raw = u16::from_str(value).map_err(|err| err.to_string())?;
        Ok(CpuAddress(raw))
    }
}
