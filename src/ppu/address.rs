const MAX_ADDRESS: u16 = 0x4000;

#[derive(Clone, Copy, Debug)]
pub struct Address(u16);

impl Address {
    pub const fn from_u16(mut value: u16) -> Option<Address> {
        if value >= MAX_ADDRESS {
            return None;
        }

        // Map the name table mirrors.
        if value >= 0x3000 && value < 0x3F00 {
            value -= 0x1000;
        }

        // Map the palette RAM index mirrors.
        if value >= 0x3F20 {
            value = 0x3F00 + value % 0x20;
        }

        Some(Address(value))
    }

    pub const fn advance(&self, value: u16) -> Address {
        let mut result = *self;
        result.0 = result.0.wrapping_add(value);
        result
    }

    pub fn to_u16(&self) -> u16 {
        self.0
    }
}
