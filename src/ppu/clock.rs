#[derive(Debug)]
pub struct Clock {
    frame: u64,
    scanline: u16,
    cycle: u16,

    total_cycles: u64,
}

impl Clock {
    pub fn new() -> Clock {
        Clock {
            frame: 0,
            scanline: 0,
            cycle: 0,

            total_cycles: 0,
        }
    }

    pub fn frame(&self) -> u64 {
        self.frame
    }

    pub fn scanline(&self) -> u16 {
        self.scanline
    }

    pub fn cycle(&self) -> u16 {
        self.cycle
    }

    pub fn total_cycles(&self) -> u64 {
        self.total_cycles
    }

    pub fn is_start_of_frame(&self) -> bool {
        self.scanline == 0 && self.cycle == 0
    }

    pub fn tick(&mut self, skip_odd_frame_cycle: bool) {
        self.total_cycles += 1;

        match (self.scanline, self.cycle) {
            (261, 340) => {
                self.frame += 1;
                self.scanline = 0;
                if skip_odd_frame_cycle && self.frame % 2 == 1 {
                    self.cycle = 1;
                } else {
                    self.cycle = 0;
                }
            },
            (_, 340) => {
                self.scanline += 1;
                self.cycle = 0;
            },
            _ => {
                self.cycle += 1;
            },
        }
    }
}