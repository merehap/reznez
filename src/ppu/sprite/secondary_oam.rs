use ux::u5;

pub struct SecondaryOam {
    data: [u8; 32],
    index: u5,
    is_full: bool,
}

impl SecondaryOam {
    pub fn new() -> SecondaryOam {
        SecondaryOam {
            data: [0xFF; 32],
            index: u5::MIN,
            is_full: false,
        }
    }

    pub fn is_full(&self) -> bool {
        self.is_full
    }

    pub fn current_field(&self) -> SpriteField {
        match u8::from(self.index) % 4 {
            0 => SpriteField::Y,
            1 => SpriteField::Tile,
            2 => SpriteField::Attributes,
            3 => SpriteField::X,
            _ => unreachable!(),
        }
    }

    pub fn peek(&self) -> u8 {
        self.data[to_usize(self.index)]
    }

    pub fn read_and_advance(&mut self) -> u8 {
        let result = self.data[to_usize(self.index)];
        self.advance();
        result
    }

    pub fn write(&mut self, value: u8) {
        self.data[to_usize(self.index)] = value;
    }

    pub fn reset_index(&mut self) {
        self.index = u5::MIN;
        self.is_full = false;
    }

    pub fn advance(&mut self) {
        if self.index == u5::MAX {
            self.index = u5::MIN;
            self.is_full = true;
        } else {
            self.index = self.index + u5::new(1);
        }
    }
}

fn to_usize(value: u5) -> usize {
    u8::from(value).into()
}

#[derive(PartialEq, Eq)]
pub enum SpriteField {
    Y,
    Tile,
    Attributes,
    X,
}