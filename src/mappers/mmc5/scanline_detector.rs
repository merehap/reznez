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

    pub fn step(&mut self, addr: PpuAddress) -> DetectedEvent {
        let prev_match_count = self.match_count;

        let region = Region::from_address(addr);
        if region != Region::NameTable && region != Region::AttributeTable {
            // Address out of range, go back to the beginning.
            self.match_count = 0;
            return DetectedEvent::Other;
        }

        let mut detected_event = if region == Region::NameTable {
            DetectedEvent::TileFetch
        } else {
            DetectedEvent::Other
        };

        // Address mismatches are ignored if match_count == 0 or a scanline has already been detected.
        if self.prev_addr != addr && matches!(self.match_count, 1 | 2) {
            // Address doesn't match the previous one, go back to the beginning.
            self.match_count = 0;
            return detected_event;
        }

        if self.match_count < 3 {
            self.match_count += 1;
        }

        self.prev_addr = addr;

        if prev_match_count == 2 && self.match_count == 3 {
            detected_event = DetectedEvent::ScanlineStart;
        }

        detected_event
    }
}

#[derive(PartialEq, Eq)]
enum Region {
    PatternTable,
    NameTable,
    AttributeTable,
    Other,
}

impl Region {
    fn from_address(addr: PpuAddress) -> Self {
        match addr.to_u16() {
            0x0000..=0x1FFF => Region::PatternTable,
            0x2000..=0x2FFF if addr.to_u16() & 0x3FF < 0x3C0 => Region::NameTable,
            // Attribute tables.
            0x2000..=0x2FFF => Region::AttributeTable,
            // Name table mirrors and the palette table.
            0x3000..=0x3FFF => Region::Other,
            0x4000..=0xFFFF => unreachable!(),
        }
    }
}

pub enum DetectedEvent {
    ScanlineStart,
    TileFetch,
    Other,
}
