use crate::ppu::ppu_clock::MAX_SCANLINE;

#[derive(Clone, Debug)]
pub struct PpuIoBus {
    value: u8,
    // Measured in scanlines.
    upper_bits_decay_countdown: u16,
    lower_bits_decay_countdown: u16,
}

impl PpuIoBus {
    pub fn new() -> PpuIoBus {
        Self {
            value: 0,
            upper_bits_decay_countdown: 0,
            lower_bits_decay_countdown: 0,
        }
    }

    pub fn value(&self) -> u8 {
        self.value
    }

    pub fn update(&mut self, value: u8) {
        self.value = value;
        // All bits are updated, both upper and lower.
        self.upper_bits_decay_countdown = MAX_SCANLINE;
        self.lower_bits_decay_countdown = MAX_SCANLINE;
    }

    pub fn update_from_status_read(&mut self, value: u8) {
        self.value = value;
        // The lower bits are unaffected since PPUStatus doesn't power them.
        self.upper_bits_decay_countdown = MAX_SCANLINE;
    }

    pub fn maybe_decay(&mut self) {
        let v = &mut self.value;
        maybe_decay_segment(v, &mut self.upper_bits_decay_countdown, 0b0001_1111);
        maybe_decay_segment(v, &mut self.lower_bits_decay_countdown, 0b1110_0000);
    }
}

#[inline]
fn maybe_decay_segment(latch: &mut u8, scanlines_remaining: &mut u16, mask: u8) {
    *scanlines_remaining = scanlines_remaining.saturating_sub(1);
    if *scanlines_remaining == 0 {
        *latch &= mask;
    }
}
