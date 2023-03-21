use std::fmt;

use crate::ppu::pixel_index::PixelRow;

pub const MAX_SCANLINE: u16 = 261;
pub const MAX_CYCLE: u16 = 340;

#[derive(Clone, Copy, Debug)]
pub struct Clock {
    frame: i64,
    scanline: u16,
    cycle: u16,

    total_cycles: u64,
}

impl Clock {
    pub fn mesen_compatible() -> Clock {
        Clock { frame: 0, scanline: 0, cycle: 6, total_cycles: 0 }
    }

    pub fn starting_at(frame: i64, scanline: u16, cycle: u16) -> Clock {
        Clock { frame, scanline, cycle, total_cycles: 0 }
    }

    pub fn frame(&self) -> i64 {
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

    pub fn tick(&mut self, skip_odd_frame_cycle: bool) -> bool {
        self.total_cycles += 1;

        let max_cycle = if skip_odd_frame_cycle && self.frame % 2 == 1 {
            MAX_CYCLE - 1
        } else {
            MAX_CYCLE
        };
        let is_last_cycle_of_frame = self.scanline == MAX_SCANLINE && self.cycle >= max_cycle;
        if is_last_cycle_of_frame {
            self.frame += 1;
            self.scanline = 0;
            self.cycle = 0;
        } else if self.cycle == MAX_CYCLE {
            self.scanline += 1;
            self.cycle = 0;
        } else {
            self.cycle += 1;
        }

        is_last_cycle_of_frame
    }
}

impl fmt::Display for Clock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "F:{},S:{:03},C:{:03}", self.frame, self.scanline, self.cycle)
    }
}
