// The smallest value for oam_stress to pass. Eight frames is almost 134 ms,
// which is 100x how long the wiki says OAM will retain its values.
const OAM_DECAY_TICKS: u8 = 8;

#[derive(Clone, Copy, Debug)]
pub struct DramByte {
    value: u8,
    // Zeros in the mask are zeros in the peeked/read value, not open bus.
    mask: u8,
    ticks_until_decay: u8,
    decay_value: u8,
}

impl DramByte {
    pub fn new() -> Self {
        Self {
            value: 0,
            mask: 0b1111_1111,
            ticks_until_decay: 0,
            decay_value: 0,
        }
    }

    pub fn with_mask(mut self, mask: u8) -> Self {
        self.mask = mask;
        self
    }

    pub fn with_decay_value(mut self, decay_value: u8) -> Self {
        self.decay_value = decay_value;
        self
    }

    pub fn peek(self) -> u8 {
        self.value
    }

    pub fn read(&mut self) -> u8 {
        self.ticks_until_decay = OAM_DECAY_TICKS;
        self.value
    }

    pub fn write(&mut self, value: u8) {
        self.ticks_until_decay = OAM_DECAY_TICKS;
        self.set(value);
    }

    fn set(&mut self, value: u8) {
        // Only store values in their already-masked state.
        self.value = value & self.mask;
    }

    pub fn maybe_decay(&mut self) {
        self.ticks_until_decay = self.ticks_until_decay.saturating_sub(1);
        if self.ticks_until_decay == 0 {
            self.set(self.decay_value);
        }
    }
}