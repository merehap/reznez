use crate::ppu::palette::rgb::Rgb;

const WIDTH: usize = 256;
const HEIGHT: usize = 240;

pub struct Screen([[Rgb; WIDTH]; HEIGHT]);

impl Screen {
    pub fn new() -> Screen {
        Screen([[Rgb::BLACK; WIDTH]; HEIGHT])
    }

    pub fn pixel(&self, column: u8, row: u8) -> Rgb {
        self.0[row as usize][column as usize]
    }

    pub fn set_pixel(&mut self, column: u8, row: u8, rgb: Rgb) {
        self.0[row as usize][column as usize] = rgb;
    }
}
