#[derive(Clone, Copy, Debug)]
pub struct DramByte {
    value: u8,
    // Zeros in the mask are zeros in the peeked/read value, not open bus.
    mask: u8,
}

impl DramByte {
    pub fn with_mask(mask: u8) -> Self {
        Self { value: 0, mask }
    }

    pub fn peek(self) -> u8 {
        self.value
    }

    pub fn write(&mut self, value: u8) {
        // Only store values in their already-masked state.
        self.value = value & self.mask;
    }
}