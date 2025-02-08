use std::fmt;

const TABLE: [u8; 0x20] = [
     10, 254,  20,   2,  40,   4,  80,   6, 160,   8,  60,  10,  14,  12,  26,  14,
     12,  16,  24,  18,  48,  20,  96,  22, 192,  24,  72,  26,  16,  28,  32,  30,
];

#[derive(PartialEq, Eq, Clone, Copy, Debug, Default)]
pub struct LengthCounter {
    count: u8,
    pending_count: Option<u8>,
    count_decremented: bool,

    halt: bool,
    pending_halt_value: Option<bool>,
}

impl LengthCounter {
    // Write $4000 (pulse 1), $4004 (pulse 2), 0x4008 (triangle) or 0x400C (noise).
    pub fn start_halt(&mut self, halt: bool) {
        self.pending_halt_value = Some(halt);
    }

    // Write $4003 (pulse 1), $4007 (pulse 2), 0x400B (triangle) or 0x400F (noise).
    pub fn start_reload(&mut self, index: u8) {
        self.pending_count = Some(TABLE[usize::from(index)]);
        self.count_decremented = false;
    }

    pub fn decrement_towards_zero(&mut self) {
        if !self.halt && self.count > 0 {
            self.count -= 1;
            self.count_decremented = true;
        }
    }

    // There's a one CPU cycle delay between the CPU writes for reloading and halting the
    // LengthCounter and the effects actually taking place. This means the LengthCounter may be
    // decremented in the interim.
    pub fn apply_pending_values(&mut self) {
        if let Some(pending_count) = self.pending_count.take() {
            // * 11-len_reload_timing.nes: "Reload during length clock when ctr > 0 should be ignored".
            // * Additionally, we assume that "Reload during length clock when HALTED should be ignored",
            // though there is no existing test that verifies this (and it's undocumented).
            // * The behavior here differs from Mesen in that pending_count can be zero. Again,
            // there's no existing test that verifies which way is correct.
            if !self.count_decremented {
                self.count = pending_count;
            }
        }

        if let Some(pending_halt_value) = self.pending_halt_value.take() {
            self.halt = pending_halt_value;
        }
    }

    pub fn is_zero(self) -> bool {
        self.count == 0
    }

    pub fn set_to_zero(&mut self) {
        self.count = 0;
    }

    fn status(self) -> Status {
        match (self.halt, self.pending_halt_value) {
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
