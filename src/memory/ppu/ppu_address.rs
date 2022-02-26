use std::fmt;

use crate::ppu::name_table::name_table_number::NameTableNumber;

const HIGH_BYTE_MASK: u16 = 0b0111_1111_0000_0000;
const LOW_BYTE_MASK:  u16 = 0b0000_0000_1111_1111;

const FINE_Y_MASK:                u16 = 0b0111_0000_0000_0000;
const VERTICAL_NAME_TABLE_MASK:   u16 = 0b0000_1000_0000_0000;
const HORIZONTAL_NAME_TABLE_MASK: u16 = 0b0000_0100_0000_0000;
const COARSE_Y_MASK:              u16 = 0b0000_0011_1110_0000;
const COARSE_X_MASK:              u16 = 0b0000_0000_0001_1111;

const Y_MASK: u16 = FINE_Y_MASK | COARSE_Y_MASK;
const NAME_TABLE_MASK: u16 = VERTICAL_NAME_TABLE_MASK | HORIZONTAL_NAME_TABLE_MASK;

const FINE_X_MASK: u8 = 0b0000_0111;

#[derive(Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub struct PpuAddress(u16, FineXScroll);

impl PpuAddress {
    pub const fn from_u16(value: u16) -> PpuAddress {
        PpuAddress(value & 0x3FFF, FineXScroll(0))
    }

    pub fn advance(&mut self, offset: u16) {
        self.0 = self.0.wrapping_add(offset) & 0x3FFF;
    }

    pub fn subtract(&mut self, offset: u16) {
        self.0 = self.0.wrapping_sub(offset) & 0x3FFF;
    }

    pub fn increment_coarse_x_scroll(&mut self) {
        if self.0 & COARSE_X_MASK == COARSE_X_MASK {
            self.0 ^= HORIZONTAL_NAME_TABLE_MASK;
            self.0 &= !COARSE_X_MASK;
        } else {
            self.0 += 1;
        }
    }

    pub fn name_table_number(self) -> NameTableNumber {
        NameTableNumber::from_last_two_bits((self.0 >> 10) as u8)
    }

    /*
     * 0123456789ABCDEF
     * -----------01234  $SCROLL#1
     * ----67----------  $CTRL
     */
    pub fn x_scroll(self) -> u8 {
        let coarse_x = ((self.0 & COARSE_X_MASK) as u8) << 3;
        let fine_x = self.1.0;
        coarse_x | fine_x
    }

    /*
     * 0123456789ABCDEF
     *  567--01234-----  $SCROLL#2
     *  ---67----------  $CTRL
     */
    pub fn y_scroll(self) -> u8 {
        let coarse_y = (self.0 & COARSE_Y_MASK) >> 2;
        let fine_y = (self.0 & FINE_Y_MASK) >> 12;
        (coarse_y | fine_y) as u8
    }

    pub fn set_name_table_number(&mut self, value: u8) {
        self.0 &= !NAME_TABLE_MASK;
        self.0 |= (value as u16 & 0b0000_0011) << 10;
    }

    pub fn set_x_scroll(&mut self, value: u8) {
        self.1 = FineXScroll(value & FINE_X_MASK);

        self.0 &= !COARSE_X_MASK;
        self.0 |= (value as u16) >> 3
    }

    pub fn set_y_scroll(&mut self, value: u8) {
        self.0 &= !Y_MASK;
        self.0 |= (value as u16 & 0b1111_1000) << 2;
        self.0 |= (value as u16 & 0b0000_0111) << 12;
    }

    pub fn set_high_byte(&mut self, value: u8) {
        self.0 &= !HIGH_BYTE_MASK;
        self.0 |= (value as u16 & 0b0011_1111) << 8;
    }

    pub fn set_low_byte(&mut self, value: u8) {
        self.0 &= !LOW_BYTE_MASK;
        self.0 |= value as u16;
    }

    pub fn to_u16(self) -> u16 {
        self.0 & 0x3FFF
    }

    pub fn to_usize(self) -> usize {
        usize::from(self.to_u16())
    }
}

impl PartialEq for PpuAddress {
    fn eq(&self, rhs: &PpuAddress) -> bool {
        self.to_u16() & 0x3FFF == rhs.to_u16() & 0x3FFF
    }
}

impl fmt::Display for PpuAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "${:04X}", self.0)
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
struct FineXScroll(u8);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reduce_bit_1_from_high_byte() {
        let mut address = PpuAddress::from_u16(0);
        address.set_high_byte(0b1111_1111);
        assert_eq!(address.0, 0b0011_1111_0000_0000);
        assert_eq!(address.to_u16(), 0b0011_1111_0000_0000);
    }

    #[test]
    fn reduce_bit_1_from_y_scroll() {
        let mut address = PpuAddress::from_u16(0);
        address.set_y_scroll(0b1111_1111);
        assert_eq!(address.0, 0b0111_0011_1110_0000);
        assert_eq!(address.to_u16(), 0b0011_0011_1110_0000);
    }

    #[test]
    fn wrap_advance() {
        let mut address = PpuAddress::from_u16(0x3FFF);
        address.advance(1);
        assert_eq!(address.0, 0x0000);
        assert_eq!(address.to_u16(), 0x0000);
    }

    #[test]
    fn set_x_scroll() {
        let mut address = PpuAddress::from_u16(0);
        assert_eq!(address.x_scroll(), 0x00);
        address.set_x_scroll(0xFF);
        assert_eq!(address.x_scroll(), 0xFF);
    }

    #[test]
    fn set_y_scroll() {
        let mut address = PpuAddress::from_u16(0);
        assert_eq!(address.y_scroll(), 0x00);
        address.set_y_scroll(0xFF);
        assert_eq!(address.y_scroll(), 0xFF);
    }

    #[test]
    fn set_x_y_scroll() {
        let mut address = PpuAddress::from_u16(0);
        assert_eq!(address.x_scroll(), 0x00);
        assert_eq!(address.y_scroll(), 0x00);
        address.set_x_scroll(0xFD);
        address.set_y_scroll(0xFF);
        assert_eq!(address.x_scroll(), 0xFD);
        assert_eq!(address.y_scroll(), 0xFF);
    }
}
