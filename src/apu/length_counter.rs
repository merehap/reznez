const TABLE: [u8; 0x20] = [
     10, 254,  20,   2,  40,   4,  80,   6, 160,   8,  60,  10,  14,  12,  26,  14,
     12,  16,  24,  18,  48,  20,  96,  22, 192,  24,  72,  26,  16,  28,  32,  30,
];

#[derive(PartialEq, Eq, Clone, Copy, Debug, Default)]
pub struct LengthCounter {
    count: u8,
    halt: bool,
}

impl LengthCounter {
    pub fn set_count_from_lookup(&mut self, index: u8) {
        self.count = TABLE[usize::from(index)];
    }

    pub fn is_zero(self) -> bool {
        self.count == 0
    }

    pub fn set_to_zero(&mut self) {
        self.count = 0;
    }

    pub fn decrement_towards_zero(&mut self) {
        if !self.halt && self.count != 0 {
            self.count -= 1;
        }
    }

    pub fn set_halt(&mut self, halt: bool) {
        self.halt = halt;
    }
}
