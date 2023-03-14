use std::fmt;

use crate::ppu::pixel_index::PixelRow;

pub const MAX_SCANLINE: u16 = 261;
pub const MAX_CYCLE: u16 = 340;

#[derive(Debug)]
pub struct Clock {
    frame: u64,
    scanline: u16,
    cycle: u16,

    total_cycles: u64,
}

impl Clock {
    pub fn new() -> Clock {
        Clock { frame: 0, scanline: 0, cycle: 0, total_cycles: 0 }
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

    pub fn scanline_pixel_row(&self) -> Option<PixelRow> {
        PixelRow::try_from_u16(self.scanline)
    }

    #[inline]
    pub fn is_last_cycle_of_frame(&self, skip_odd_frame_cycle: bool) -> bool {
        let max_cycle = if skip_odd_frame_cycle && self.frame % 2 == 1 {
            MAX_CYCLE - 1
        } else {
            MAX_CYCLE
        };
        self.scanline == MAX_SCANLINE && self.cycle >= max_cycle
    }

    pub fn tick(&mut self, skip_odd_frame_cycle: bool) {
        self.total_cycles += 1;

        match (self.scanline, self.cycle) {
            (_, _) if self.is_last_cycle_of_frame(skip_odd_frame_cycle) => {
                self.frame += 1;
                self.scanline = 0;
                self.cycle = 0;
            }
            (_, MAX_CYCLE) => {
                self.scanline += 1;
                self.cycle = 0;
            }
            _ => {
                self.cycle += 1;
            }
        }
    }
}

impl fmt::Display for Clock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "F:{},S:{:03},C:{:03}", self.frame, self.scanline, self.cycle)
    }
}
