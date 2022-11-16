pub struct SecondaryOam {
    data: [u8; 32],
    index: usize,
    is_full: bool,
}

impl SecondaryOam {
    pub fn new() -> SecondaryOam {
        SecondaryOam {
            data: [0xFF; 32],
            index: 0,
            is_full: false,
        }
    }

    pub fn is_full(&self) -> bool {
        self.is_full
    }

    pub fn read_and_advance(&mut self) -> u8 {
        let result = self.data[self.index];
        self.advance();
        result
    }

    pub fn write(&mut self, value: u8) {
        if !self.is_full {
            self.data[self.index] = value;
        }
    }

    pub fn reset_index(&mut self) {
        self.index = 0;
        self.is_full = false;
    }

    pub fn advance(&mut self) {
        if self.index == 31 {
            self.index = 0;
            self.is_full = true;
        } else {
            self.index += 1;
        }
    }
}
