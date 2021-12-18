#[derive(Clone, Copy, Debug)]
pub struct Rgb {
    red: u8,
    green: u8,
    blue: u8,
}

impl Rgb {
    pub const BLACK: Rgb = Rgb::new(0x00, 0x00, 0x00);
    pub const WHITE: Rgb = Rgb::new(0xFF, 0xFF, 0xFF);

    pub const fn new(red: u8, green: u8, blue: u8) -> Rgb {
        Rgb {red, green, blue}
    }

    pub fn red(self) -> u8 {
        self.red
    }

    pub fn green(self) -> u8 {
        self.green
    }

    pub fn blue(self) -> u8 {
        self.blue
    }
}
