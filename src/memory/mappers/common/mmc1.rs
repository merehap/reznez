const EMPTY_SHIFT_REGISTER: u8 = 0b0001_0000;

pub struct ShiftRegister {
    value: u8,
}

impl ShiftRegister {
    pub fn shift(&mut self, write_value: u8) -> ShiftStatus {
        if write_value & 0b1000_0000 != 0 {
            self.value = EMPTY_SHIFT_REGISTER;
            return ShiftStatus::Clear;
        }

        let is_last_shift = self.value & 1 == 1;
        self.value >>= 1;
        // Copy the last bit from write_value to the front of self.value.
        self.value |= (write_value & 1) << 4;

        if !is_last_shift {
            return ShiftStatus::Continue;
        }

        let finished_value = self.value;
        self.value = EMPTY_SHIFT_REGISTER;
        ShiftStatus::Done { finished_value }
    }
}

impl Default for ShiftRegister {
    fn default() -> Self {
        Self { value: EMPTY_SHIFT_REGISTER }
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum ShiftStatus {
    Clear,
    Continue,
    Done { finished_value: u8 },
}
