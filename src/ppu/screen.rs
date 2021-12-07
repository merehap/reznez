use crate::ppu::palette::rgb::Rgb;

pub struct Screen([[Rgb; Screen::WIDTH]; Screen::HEIGHT]);

impl Screen {
    pub const WIDTH: usize = 256;
    pub const HEIGHT: usize = 240;

    pub fn new() -> Screen {
        Screen([[Rgb::BLACK; Screen::WIDTH]; Screen::HEIGHT])
    }

    pub fn pixel(&self, column: u8, row: u8) -> Rgb {
        self.0[row as usize][column as usize]
    }

    pub fn set_pixel(&mut self, column: u8, row: u8, rgb: Rgb) {
        self.0[row as usize][column as usize] = rgb;
    }
}
