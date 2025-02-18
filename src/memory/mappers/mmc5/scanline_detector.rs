use crate::memory::ppu::ppu_address::PpuAddress;

// Determines when a new scanline is detected, if rendering is enabled.
// Indicates that the "in frame" state may occur when scanline_detected() is true.
pub struct ScanlineDetector {
    // How many PPU reads in a row have had the same in-range address.
    match_count: u8,
    // The previous PPU address read.
    prev_addr: PpuAddress,
}

impl ScanlineDetector {
    pub fn new() -> Self {
        Self {
            // MMC5 powers up with the assumption that a scanline is detected:
            // https://www.nesdev.org/wiki/File:Mmc5_in_frame_status_bit.png
            match_count: 3,
            prev_addr: PpuAddress::ZERO,
        }
    }

    pub fn scanline_detected(&self) -> bool {
        self.match_count >= 3
    }

    pub fn step(&mut self, addr: PpuAddress) -> bool {
        let prev_match_count = self.match_count;

        let is_in_name_table = matches!(addr.to_u16(), 0x2000..=0x2FFF);
        // If match_count == 0 or a scanline has been detected, then ignore address mismatches.
        let mismatched_addrs = matches!(self.match_count, 1 | 2) && self.prev_addr != addr;
        if !is_in_name_table || mismatched_addrs {
            // Address out of range or doesn't match the previous one, go back to the beginning.
            self.match_count = 0;
            // No scanline detected.
            return false;
        }

        if self.match_count < 3 {
            self.match_count += 1;
        }

        self.prev_addr = addr;

        let new_scanline_detected = prev_match_count == 2 && self.match_count == 3;
        new_scanline_detected
    }
}
