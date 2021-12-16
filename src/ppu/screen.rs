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

    pub fn tile_sliver(&mut self, column: u8, row: u8) -> &mut [Rgb; 8] {
        let row = &mut self.0[row as usize];
        let column = &mut row[column as usize..column as usize + 8];
        column
            .try_into()
            .unwrap()
    }
}
