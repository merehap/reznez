// Some memory sources do not power every bit in their bytes, instead leaving them as always zero.
#[derive(Clone, Copy, Debug)]
pub struct MaskedByte<const MASK: u8>(u8);

impl <const MASK: u8> MaskedByte<MASK> {
    pub const ZERO: Self = Self(0);

    pub fn new(unmasked_byte: u8) -> Self {
        let mut result = Self::ZERO;
        result.write(unmasked_byte);
        result
    }

    pub fn peek(self) -> u8 {
        self.0
    }

    pub fn write(&mut self, unmasked_byte: u8) {
        self.0 = unmasked_byte & MASK;
    }
}