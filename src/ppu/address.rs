const MAX_ADDRESS: u16 = 0x4000;

pub struct Address(u16);

impl Address {
    pub fn from_u16(mut value: u16) -> Result<Address, String> {
        if value >= MAX_ADDRESS {
            return Err(format!("PPU Addresses must be under {:X}", MAX_ADDRESS));
        }

        // Map the name table mirrors.
        if value >= 0x3000 && value < 0x3F00 {
            value -= 0x1000;
        }

        // Map the palette RAM index mirrors.
        if value >= 0x3F20 {
            value = 0x3F00 + value % 0x20;
        }

        Ok(Address(value))
    }

    pub fn to_u16(&self) -> u16 {
        self.0
    }
}
