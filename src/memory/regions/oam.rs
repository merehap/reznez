use std::fmt;

use itertools::Itertools;

use crate::memory::primitives::dram_byte::DramByte;
use crate::ppu::ppu_clock::PpuClock;
use crate::ppu::sprite::oam_address::OamAddress;

#[derive(Clone)]
pub struct Oam([DramByte; 256]);

impl Oam {
    pub fn new() -> Oam {
        let sprite = [
            DramByte::new().with_mask(0b1111_1111).with_decay_value(0b1111_1111),
            DramByte::new().with_mask(0b1111_1111).with_decay_value(0b1111_1111),
            DramByte::new().with_mask(0b1110_0011).with_decay_value(0b1110_0011), // Sprite attribute byte, normally
            DramByte::new().with_mask(0b1111_1111).with_decay_value(0b1111_1111),
        ];

        // 64 sprites, 256 bytes
        let raw_oam = [&sprite[..]; 64].concat();
        Oam(raw_oam.try_into().unwrap())
    }

    // For debug screens only.
    pub fn to_raw(&self) -> &[DramByte; 256] {
        &self.0
    }

    pub fn peek(&self, clock: &PpuClock, oam_addr: OamAddress, rendering_enabled: bool) -> u8 {
        let mut value = self.0[oam_addr.to_u8() as usize].peek();
        if (clock.is_oam_clearing_window() || clock.is_secondary_oam_transfer_window()) && rendering_enabled {
            value = 0xFF;
        }

        value
    }

    pub fn read(&mut self, clock: &PpuClock, oam_addr: OamAddress, rendering_enabled: bool) -> u8 {
        // TODO: Verify that the read still actually happens if oam is being cleared, since the value is discarded.
        let mut value = self.0[oam_addr.to_u8() as usize].read();
        if (clock.is_oam_clearing_window() || clock.is_secondary_oam_transfer_window()) && rendering_enabled {
            value = 0xFF;
        }

        value
    }

    pub fn write(&mut self, address: OamAddress, value: u8) {
        self.0[address.to_u8() as usize].write(value);
    }

    pub fn maybe_corrupt_starting_byte(&mut self, clock: &PpuClock, oam_addr: OamAddress, rendering_enabled: bool) {
        let index = u8::try_from(clock.cycle()).unwrap() - 1;
        let raw_address = oam_addr.to_u8();
        if raw_address >= 0x08 {
            let corrupted_addr = OamAddress::from_u8((raw_address & 0xF8) + index);
            // TODO: Should this be read() instead of peek()? Probably.
            let value = self.peek(clock, corrupted_addr, rendering_enabled);
            self.write(OamAddress::from_u8(index), value);
        }
    }

    pub fn maybe_decay(&mut self) {
        for dram_byte in self.0.iter_mut() {
            dram_byte.maybe_decay();
        }
    }
}

impl fmt::Display for Oam {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for chunk in &self.0.iter().chunks(16) {
            for value in chunk {
                write!(f, "{:02X} ", value.peek())?;
            }

            writeln!(f)?;
        }

        Ok(())
    }
}