#[derive(Clone, Copy)]
pub struct Address(u16);

impl Address {
    pub fn new(mut value: u16) -> Address {
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
        Address::new(((high as u16) << 8) + (low as u16))
    }

    pub fn zero_page(low: u8) -> Address {
        Address::new(low as u16)
    }

    pub fn to_raw(&self) -> u16 {
        self.0
    }

    pub fn advance(&self, value: u8) -> Address {
        let mut result = *self;
        result.0 = result.0.wrapping_add(value as u16);
        result
    }

    pub fn offset(&self, value: i8) -> Address {
        let mut result = *self;
        result.0 = (result.0 as i32).wrapping_add(value as i32) as u16;
        result
    }

    pub fn get_type(&self) -> AddressType {
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

pub enum AddressType {
    InternalRAM,
    PpuRegister,
    ApuRegister,
    DisabledApuRegister,
    Cartridge,
    InterruptVector,
}
