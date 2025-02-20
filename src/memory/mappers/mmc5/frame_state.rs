use crate::memory::mappers::mmc5::scanline_detector::{ScanlineDetector, DetectedEvent};
use crate::memory::ppu::ppu_address::PpuAddress;

const SPRITE_PATTERN_FETCH_START: u8 = 64;
const BACKGROUND_PATTERN_FETCH_START: u8 = 81;

pub struct FrameState {
    in_frame: bool,
    irq_pending: bool,

    scanline_detector: ScanlineDetector,
    irq_target_scanline: u8,
    scanline: u8,

    ppu_is_reading: bool,
    idle_count: u8,
    pattern_fetch_count: u8,
}

impl FrameState {
    pub fn new() -> Self {
        Self {
            in_frame: false,
            irq_pending: false,

            scanline_detector: ScanlineDetector::new(),
            // A target of 0 means IRQs are disabled (except an already pending one).
            irq_target_scanline: 0,
            scanline: 0,

            ppu_is_reading: false,
            idle_count: 0,
            pattern_fetch_count: 0,
        }
    }

    pub fn in_frame(&self) -> bool {
        self.in_frame
    }

    pub fn irq_pending(&self) -> bool {
        self.irq_pending
    }

    pub fn sprite_fetching(&self) -> bool {
        (SPRITE_PATTERN_FETCH_START..BACKGROUND_PATTERN_FETCH_START)
            .contains(&self.pattern_fetch_count)
    }

    // Called on PPU mask (0x2001) write, and on NMI vector (0xFFFA or 0xFFFB) read.
    pub fn force_end_frame(&mut self) {
        self.in_frame = false;
    }

    // Called on 0x5203 write.
    pub fn set_target_irq_scanline(&mut self, target: u8) {
        self.irq_target_scanline = target;
    }

    // Called on 0x5204 read, and on NMI vector (0xFFFA or 0xFFFB) read.
    pub fn acknowledge_irq(&mut self) {
        self.irq_pending = false;
    }

    // Called every PPU read.
    pub fn sync_frame_status(&mut self, addr: PpuAddress) {
        self.ppu_is_reading = true;

        match self.scanline_detector.step(addr) {
            // A new frame is starting.
            DetectedEvent::ScanlineStart if !self.in_frame => {
                self.in_frame = true;
                self.scanline = 0;
                self.irq_pending = false;
                self.pattern_fetch_count = 0;
            }
            // A new scanline is starting in the ongoing frame.
            DetectedEvent::ScanlineStart => {
                self.scanline += 1;
                if self.scanline == self.irq_target_scanline {
                    self.irq_pending = true;
                }

                self.pattern_fetch_count = 0;
            }
            // A new pattern was read.
            DetectedEvent::PatternFetch => {
                self.pattern_fetch_count += 1;
            }
            // Nothing interesting happened.
            DetectedEvent::Other => {}
        }
    }

    // Called every CPU cycle.
    pub fn maybe_end_frame(&mut self) {
        if self.ppu_is_reading {
            self.ppu_is_reading = false;
            self.idle_count = 0;
            return;
        }

        if self.idle_count < 3 {
            self.idle_count += 1;
        }

        if self.idle_count == 3 {
            // No PPU reads occurred over 3 CPU cycles so rendering must be disabled: end the frame.
            self.in_frame = false;
        }
    }
}
