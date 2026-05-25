use std::fmt;

use crate::ppu::pixel_index::PixelRow;

pub const MAX_SCANLINE: u16 = 261;

#[derive(Clone, Debug)]
pub struct PpuClock {
    frame: i64,
    scanline: u16,
    cycle: u16,

    total_cycles: u64,
}

impl PpuClock {
    pub fn mesen_compatible() -> PpuClock {
        PpuClock { frame: 0, scanline: 0, cycle: 5, total_cycles: 0 }
    }

    pub fn starting_at(frame: i64, scanline: u16, cycle: u16) -> PpuClock {
        PpuClock { frame, scanline, cycle, total_cycles: 0 }
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
        PixelRow::from_scanline(self.scanline)
    }

    pub fn is_on_visible_scanline(&self) -> bool {
        self.scanline < 240
    }

    pub fn is_on_vblank_scanline(&self) -> bool {
        self.scanline >= 240 && self.scanline < 261
    }

    pub fn is_on_prerender_scanline(&self) -> bool {
        self.scanline == 261
    }

    pub fn is_oam_clearing_window(&self) -> bool {
        // TODO: Should the prerender scanline be included here, too?
        self.is_on_visible_scanline() && self.cycle >= 1 && self.cycle <= 64
    }

    pub fn is_secondary_oam_transfer_window(&self) -> bool {
        // TODO: Should the prerender scanline be included here, too?
        self.is_on_visible_scanline() && self.cycle >= 256 && self.cycle <= 320
    }

    pub fn tick(&mut self, skip_odd_frame_cycle: bool) -> Option<LastCycle> {
        self.total_cycles += 1;

        let last_cycle = if skip_odd_frame_cycle && self.frame % 2 == 1 {
            LastCycle::Skipped
        } else {
            LastCycle::Normal
        };

        let is_last_cycle_of_frame = self.scanline == MAX_SCANLINE && self.cycle >= last_cycle as u16;
        if is_last_cycle_of_frame {
            self.frame += 1;
            self.scanline = 0;
            self.cycle = 0;
            Some(last_cycle)
        } else if self.cycle == LastCycle::Normal as u16 {
            self.scanline += 1;
            self.cycle = 0;
            None
        } else {
            self.cycle += 1;
            None
        }
    }
}

impl fmt::Display for PpuClock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "F:{},S:{:03},C:{:03}", self.frame, self.scanline, self.cycle)
    }
}

#[derive(Clone, Copy)]
pub enum LastCycle {
    Normal = 340,
    Skipped = 339,
}
