use crate::util::get_bit;

pub struct Tile<'a> {
    bytes: &'a [u8; 16],
}

impl <'a> Tile<'a> {
    pub fn new(bytes: &'a [u8; 16]) -> Tile<'a> {
        Tile {bytes}
    }

    pub fn pixel_at(&self, column: usize, row: usize) -> Pixel {
        let low_bit = get_bit(self.bytes[row], column);
        let high_bit = get_bit(self.bytes[row + 8], column);
        match (low_bit, high_bit) {
            (false, false) => Pixel::Transparent,
            (true , false) => Pixel::One,
            (false, true ) => Pixel::Two,
            (true , true ) => Pixel::Three,
        }
    }
}

pub enum Pixel {
    Transparent,
    One,
    Two,
    Three,
}
