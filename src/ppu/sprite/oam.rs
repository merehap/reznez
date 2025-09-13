use crate::ppu::sprite::oam_address::OamAddress;

const ATTRIBUTE_BYTE_INDEX: u8 = 2;

// TODO: OAM should decay:
// https://wiki.nesdev.org/w/index.php?title=PPU_OAM#Dynamic_RAM_decay
#[derive(Clone)]
pub struct Oam([u8; 256]);

impl Oam {
    pub fn new() -> Oam {
        Oam([0; 256])
    }

    pub fn to_bytes(&self) -> &[u8; 256] {
        &self.0
    }

    // FIXME: For debug screens only. Figure out how to change the debug screen to not require mutability.
    pub fn to_bytes_mut(&mut self) -> &mut [u8; 256] {
        &mut self.0
    }

    pub fn peek(&self, address: OamAddress) -> u8 {
        self.0[address.to_u8() as usize]
    }

    pub fn write(&mut self, address: OamAddress, value: u8) {
        let address = address.to_u8();
        // The three unimplemented attribute bits should never be set.
        // FIXME: Use method, not mod.
        let value = if address % 4 == ATTRIBUTE_BYTE_INDEX {
            value & 0b1110_0011
        } else {
            value
        };
        self.0[address as usize] = value;
    }

    pub fn maybe_corrupt_starting_byte(&mut self, address: OamAddress, cycle: u16) {
        let index = cycle as usize - 1;
        let address = address.to_u8();
        if address >= 0x08 {
            self.0[index] = self.0[(address & 0xF8) as usize + index];
        }
    }
}