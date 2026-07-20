use std::fmt;

use crate::{ppu::pixel_index::PixelRow, signals::Signals};

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

    pub fn tick(&mut self, signals: &mut Signals) -> Option<LastCycle> {
        self.total_cycles += 1;

        let skip_odd_frame_cycle = signals.rendering_enabled;
        let last_cycle = if skip_odd_frame_cycle && self.frame % 2 == 1 {
            LastCycle::Skipped
        } else {
            LastCycle::Normal
        };

        let old_scanline = self.scanline;

        let is_last_cycle_of_frame = self.scanline == MAX_SCANLINE && self.cycle >= last_cycle as u16;
        let last_cycle = if is_last_cycle_of_frame {
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
        };

        if self.scanline != old_scanline {
            match self.scanline {
                0 => signals.visible_scanlines_starting(),
                240 => signals.vblank_scanlines_starting(),
                261 => signals.prerender_scanline_starting(),
                _ => { /* Continue with the current scanline type. */ }
            }
        }

        match self.cycle {
            1 => signals.oam_clearing_starting(),
            65 => signals.other_sprite_rendering_dot_type_starting(),
            257 => signals.secondary_oam_transfer_starting(),
            321 => signals.other_sprite_rendering_dot_type_starting(),
            _ => { /* Continue with the current dot type. */ }
        }

        last_cycle
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
