use std::fmt;

#[derive(Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub struct PpuAddress(u16);

impl PpuAddress {
    pub const fn from_u16(value: u16) -> PpuAddress {
        PpuAddress(value)
    }

    pub const fn advance(self, offset: u16) -> PpuAddress {
        PpuAddress::from_u16(self.0.wrapping_add(offset))
    }

    pub fn to_u16(self) -> u16 {
        let value = self.0;
        match value {
            // Mirrors of lower memory (0x0000 through 0x3FFF).
            0x4000..=0xFFFF => value & 0x3FFF,
            _ => value,
        }
    }

    pub fn to_usize(self) -> usize {
        usize::from(self.to_u16())
    }
}

impl PartialEq for PpuAddress {
    fn eq(&self, rhs: &PpuAddress) -> bool {
        self.to_u16() == rhs.to_u16()
    }
}

impl fmt::Display for PpuAddress {
    fn fmt<'a>(&self, f: &mut std::fmt::Formatter<'a>) -> fmt::Result {
        write!(f, "${:04X}", self.0)
    }
}
