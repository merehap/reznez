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
        Rgb { red, green, blue }
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

    pub fn to_greyscale(self) -> Self {
        Self {
            red: self.red & 0x30,
            green: self.green & 0x30,
            blue: self.blue & 0x30,
        }
    }

    pub fn emphasized(self, factors: [f32; 3]) -> Rgb {
        Self {
            red: apply_emphasis_factor(self.red, factors[0]),
            green: apply_emphasis_factor(self.green, factors[1]),
            blue: apply_emphasis_factor(self.blue, factors[2]),
        }
    }
}

fn apply_emphasis_factor(component: u8, factor: f32) -> u8 {
    ((component as f32) * factor).clamp(0.0, 255.0) as u8
}
