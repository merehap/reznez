use crate::ppu::clock::MAX_SCANLINE;
use crate::ppu::register::register_type::RegisterType;

#[derive(Clone, Copy)]
pub struct PpuIoBus {
    value: u8,
    scanlines_until_decay: Option<u16>,
    scanlines_until_unused_status_bits_decay: Option<u16>,
}

impl PpuIoBus {
    pub fn new() -> PpuIoBus {
        PpuIoBus {
            value: 0,
            scanlines_until_decay: None,
            scanlines_until_unused_status_bits_decay: None,
        }
    }

    pub fn value(&self) -> u8 {
        self.value
    }

    pub fn update_from_read(&mut self, register_type: RegisterType, value: u8) {
        self.value = value;

        self.scanlines_until_unused_status_bits_decay =
            if register_type == RegisterType::Status {
                // The unused status bits remain on the old decay schedule.
                self.scanlines_until_decay
            } else {
                // All bit decays are now in sync, so stop tracking this.
                None
            };

        // At least one frame should occur before the latch decays to zero.
        self.scanlines_until_decay = Some(MAX_SCANLINE);
    }

    pub fn update_from_write(&mut self, value: u8) {
        self.value = value;
        // About one frame should occur before the latch decays to zero.
        self.scanlines_until_decay = Some(MAX_SCANLINE);
        // All bit decays are now in sync, so stop tracking this.
        self.scanlines_until_unused_status_bits_decay = None;
    }

    pub fn maybe_decay(&mut self) {
        let v = &mut self.value;
        maybe_decay_internal(v, &mut self.scanlines_until_decay, 0b0000_0000);
        maybe_decay_internal(
            v,
            &mut self.scanlines_until_unused_status_bits_decay,
            0b1110_0000,
        );
    }
}

#[inline]
fn maybe_decay_internal(latch: &mut u8, scanlines_remaining: &mut Option<u16>, mask: u8) {
    match *scanlines_remaining {
        None => { /* The bits have already decayed. */ }
        Some(0) => {
            // Decay the latch and halt the decay process.
            *latch &= mask;
            *scanlines_remaining = None;
        }
        Some(scanlines) => *scanlines_remaining = Some(scanlines - 1),
    }
}
