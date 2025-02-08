use std::fmt;

const TABLE: [u8; 0x20] = [
     10, 254,  20,   2,  40,   4,  80,   6, 160,   8,  60,  10,  14,  12,  26,  14,
     12,  16,  24,  18,  48,  20,  96,  22, 192,  24,  72,  26,  16,  28,  32,  30,
];

#[derive(PartialEq, Eq, Clone, Copy, Debug, Default)]
pub struct LengthCounter {
    count: u8,
    halt: bool,
    next_halt_value: Option<bool>,
}

impl LengthCounter {
    // Write $4000 (pulse 1), $4004 (pulse 2), 0x4008 (triangle) or 0x400C (noise).
    pub fn set_halt(&mut self, halt: bool) {
        self.next_halt_value = Some(halt);
    }

    // Write $4003 (pulse 1), $4007 (pulse 2), 0x400B (triangle) or 0x400F (noise).
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
        if !self.halt && self.count > 0 {
            self.count -= 1;
        }
    }

    pub fn apply_halt(&mut self) {
        if let Some(next_halt_value) = self.next_halt_value {
            self.halt = next_halt_value;
            self.next_halt_value = None;
        }
    }

    fn status(self) -> Status {
        match (self.halt, self.next_halt_value) {
            (false, None | Some(false)) => Status::Normal,
            (true , None | Some(true) ) => Status::Halted,
            (false,        Some(true) ) => Status::HaltPending,
            (true ,        Some(false)) => Status::UnhaltPending,
        }
    }
}

#[derive(PartialEq, Eq, Debug)]
enum Status {
    Normal,
    Halted,
    HaltPending,
    UnhaltPending,
}

impl fmt::Display for LengthCounter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        if self.status() == Status::Normal {
            write!(f, "({})", self.count)
        } else {
            write!(f, "({}, {:?})", self.count, self.status())
        }
    }
}
