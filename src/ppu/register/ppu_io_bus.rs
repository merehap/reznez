use std::cell::Cell;
use std::rc::Rc;

use crate::ppu::ppu_clock::MAX_SCANLINE;

#[derive(Clone)]
pub struct PpuIoBus {
    value: Rc<Cell<u8>>,
    scanlines_until_decay: Option<u16>,
    scanlines_until_unused_status_bits_decay: Option<u16>,
}

impl PpuIoBus {
    pub fn new() -> PpuIoBus {
        PpuIoBus {
            value: Rc::new(Cell::new(0)),
            scanlines_until_decay: None,
            scanlines_until_unused_status_bits_decay: None,
        }
    }

    pub fn value(&self) -> u8 {
        self.value.get()
    }

    pub fn update_from_read(&mut self, value: u8) {
        self.value.set(value);
        // All bit decays are now in sync, so stop tracking this.
        self.scanlines_until_unused_status_bits_decay = None;
        // At least one frame should occur before the latch decays to zero.
        self.scanlines_until_decay = Some(MAX_SCANLINE);
    }

    pub fn update_from_status_read(&mut self, value: u8) {
        self.value.set(value);
        // The unused status bits remain on the old decay schedule.
        self.scanlines_until_unused_status_bits_decay = self.scanlines_until_decay;
        // At least one frame should occur before the latch decays to zero.
        self.scanlines_until_decay = Some(MAX_SCANLINE);
    }

    pub fn update_from_write(&mut self, value: u8) {
        self.value.set(value);
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
fn maybe_decay_internal(latch: &Rc<Cell<u8>>, scanlines_remaining: &mut Option<u16>, mask: u8) {
    match *scanlines_remaining {
        None => { /* The bits have already decayed. */ }
        Some(0) => {
            // Decay the latch and halt the decay process.
            latch.update(|l| l & mask);
            *scanlines_remaining = None;
        }
        Some(scanlines) => *scanlines_remaining = Some(scanlines - 1),
    }
}
