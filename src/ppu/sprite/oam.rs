use std::fmt;

use itertools::Itertools;

use crate::memory::primitives::dram_byte::DramByte;
use crate::ppu::sprite::oam_address::OamAddress;

// TODO: OAM should decay:
// https://wiki.nesdev.org/w/index.php?title=PPU_OAM#Dynamic_RAM_decay
#[derive(Clone)]
pub struct Oam([DramByte; 256]);

impl Oam {
    pub fn new() -> Oam {
        let sprite = [
            DramByte::with_mask(0b1111_1111),
            DramByte::with_mask(0b1111_1111),
            DramByte::with_mask(0b1110_0011), // Sprite attribute byte
            DramByte::with_mask(0b1111_1111),
        ];

        // 64 sprites
        let raw_oam = [
            &sprite[..], &sprite[..], &sprite[..], &sprite[..], &sprite[..], &sprite[..], &sprite[..], &sprite[..],
            &sprite[..], &sprite[..], &sprite[..], &sprite[..], &sprite[..], &sprite[..], &sprite[..], &sprite[..],
            &sprite[..], &sprite[..], &sprite[..], &sprite[..], &sprite[..], &sprite[..], &sprite[..], &sprite[..],
            &sprite[..], &sprite[..], &sprite[..], &sprite[..], &sprite[..], &sprite[..], &sprite[..], &sprite[..],
            &sprite[..], &sprite[..], &sprite[..], &sprite[..], &sprite[..], &sprite[..], &sprite[..], &sprite[..],
            &sprite[..], &sprite[..], &sprite[..], &sprite[..], &sprite[..], &sprite[..], &sprite[..], &sprite[..],
            &sprite[..], &sprite[..], &sprite[..], &sprite[..], &sprite[..], &sprite[..], &sprite[..], &sprite[..],
            &sprite[..], &sprite[..], &sprite[..], &sprite[..], &sprite[..], &sprite[..], &sprite[..], &sprite[..],
        ].concat();

        Oam(raw_oam.try_into().unwrap())
    }

    pub fn to_raw(&self) -> &[DramByte; 256] {
        &self.0
    }

    pub fn peek(&self, address: OamAddress) -> u8 {
        self.0[address.to_u8() as usize].peek()
    }

    pub fn write(&mut self, address: OamAddress, value: u8) {
        self.0[address.to_u8() as usize].write(value);
    }

    pub fn maybe_corrupt_starting_byte(&mut self, address: OamAddress, cycle: u16) {
        let index = u8::try_from(cycle).unwrap() - 1;
        let raw_address = address.to_u8();
        if raw_address >= 0x08 {
            // TODO: Should this be read() instead of peek()? Probably.
            let value = self.peek(OamAddress::from_u8((raw_address & 0xF8) + index));
            self.write(OamAddress::from_u8(index), value);
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