#[rustfmt::skip]

use std::fmt;

use num_traits::FromPrimitive;

use crate::ppu::name_table::background_tile_index::{TileColumn, TileRow};
use crate::ppu::name_table::name_table_position::NameTablePosition;
use crate::ppu::pixel_index::{ColumnInTile, PixelColumn, PixelRow, RowInTile};

const HIGH_BYTE_MASK: u16 = 0b0111_1111_0000_0000;
const LOW_BYTE_MASK: u16 = 0b0000_0000_1111_1111;

const FINE_Y_MASK: u16 = 0b0111_0000_0000_0000;
const VERTICAL_NAME_TABLE_MASK: u16 = 0b0000_1000_0000_0000;
const HORIZONTAL_NAME_TABLE_MASK: u16 = 0b0000_0100_0000_0000;
const COARSE_Y_MASK: u16 = 0b0000_0011_1110_0000;
const COARSE_X_MASK: u16 = 0b0000_0000_0001_1111;

const Y_MASK: u16 = FINE_Y_MASK | COARSE_Y_MASK;
const NAME_TABLE_MASK: u16 = VERTICAL_NAME_TABLE_MASK | HORIZONTAL_NAME_TABLE_MASK;

const FINE_X_MASK: u8 = 0b0000_0111;

#[derive(Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub struct PpuAddress {
    address: u16,
    fine_x_scroll: ColumnInTile,
}

impl PpuAddress {
    pub const fn from_u16(value: u16) -> PpuAddress {
        PpuAddress {
            address: value & 0x3FFF,
            fine_x_scroll: ColumnInTile::Zero,
        }
    }

    pub fn advance(&mut self, offset: u16) {
        self.address = self.address.wrapping_add(offset) & 0x3FFF;
    }

    pub fn subtract(&mut self, offset: u16) {
        self.address = self.address.wrapping_sub(offset) & 0x3FFF;
    }

    pub fn increment_coarse_x_scroll(&mut self) {
        if self.address & COARSE_X_MASK == COARSE_X_MASK {
            self.address ^= HORIZONTAL_NAME_TABLE_MASK;
            self.address &= !COARSE_X_MASK;
        } else {
            self.address += 1;
        }
    }

    pub fn name_table_position(self) -> NameTablePosition {
        NameTablePosition::from_last_two_bits((self.address >> 10) as u8)
    }

    /*
     * 0123456789ABCDEF
     * -----------01234  $SCROLL#1
     * ----67----------  $CTRL
     */
    pub fn x_scroll(self) -> XScroll {
        let coarse = (self.address & COARSE_X_MASK) as u8;
        XScroll {
            coarse: TileColumn::try_from_u8(coarse).unwrap(),
            fine: self.fine_x_scroll,
        }
    }

    /*
     * 0123456789ABCDEF
     *  567--01234-----  $SCROLL#2
     *  ---67----------  $CTRL
     */
    pub fn y_scroll(self) -> YScroll {
        let coarse = ((self.address & COARSE_Y_MASK) >> 5) as u8;
        let fine = ((self.address & FINE_Y_MASK) >> 12) as u8;
        YScroll {
            coarse: TileRow::try_from_u8(coarse).unwrap(),
            fine: RowInTile::from_u8(fine).unwrap(),
        }
    }

    pub fn set_name_table_position(&mut self, value: u8) {
        self.address &= !NAME_TABLE_MASK;
        self.address |= (u16::from(value) & 0b0000_0011) << 10;
    }

    pub fn set_x_scroll(&mut self, value: u8) {
        self.fine_x_scroll = ColumnInTile::from_u8(value & FINE_X_MASK).unwrap();

        self.address &= !COARSE_X_MASK;
        self.address |= u16::from(value) >> 3;
    }

    pub fn set_y_scroll(&mut self, value: u8) {
        self.address &= !Y_MASK;
        self.address |= (u16::from(value) & 0b1111_1000) << 2;
        self.address |= (u16::from(value) & 0b0000_0111) << 12;
    }

    pub fn set_high_byte(&mut self, value: u8) {
        self.address &= !HIGH_BYTE_MASK;
        self.address |= (u16::from(value) & 0b0011_1111) << 8;
    }

    pub fn set_low_byte(&mut self, value: u8) {
        self.address &= !LOW_BYTE_MASK;
        self.address |= u16::from(value);
    }

    pub fn to_u16(self) -> u16 {
        self.address & 0x3FFF
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
        write!(f, "${:04X}", self.address)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct XScroll {
    coarse: TileColumn,
    fine: ColumnInTile,
}

impl XScroll {
    pub const ZERO: XScroll =
        XScroll { coarse: TileColumn::ZERO, fine: ColumnInTile::Zero };

    pub fn coarse(self) -> TileColumn {
        self.coarse
    }

    pub fn fine(self) -> ColumnInTile {
        self.fine
    }

    pub fn is_zero(self) -> bool {
        self.coarse.to_u8() == 0 && self.fine == ColumnInTile::Zero
    }

    pub fn tile_column(self, pixel_column: PixelColumn) -> (TileColumn, ColumnInTile) {
        let offset_pixel_column = self.to_u8().wrapping_add(pixel_column.to_u8());
        (
            TileColumn::try_from_u8(offset_pixel_column / 8).unwrap(),
            ColumnInTile::from_u8(offset_pixel_column % 8).unwrap(),
        )
    }

    pub fn to_u8(self) -> u8 {
        (self.coarse.to_u8() << 3) | self.fine as u8
    }
}

#[derive(Clone, Copy, Debug)]
pub struct YScroll {
    coarse: TileRow,
    fine: RowInTile,
}

impl YScroll {
    pub const ZERO: YScroll = YScroll { coarse: TileRow::ZERO, fine: RowInTile::Zero };

    fn from_u8(value: u8) -> YScroll {
        YScroll {
            coarse: TileRow::try_from_u8(value >> 3).unwrap(),
            fine: RowInTile::from_u8(value & 0b111).unwrap(),
        }
    }

    pub fn shift_down(self) -> YScroll {
        YScroll::from_u8(self.to_u8().wrapping_sub(240))
    }

    pub fn coarse(self) -> TileRow {
        self.coarse
    }

    pub fn fine(self) -> RowInTile {
        self.fine
    }

    pub fn is_zero(self) -> bool {
        self.coarse.to_u8() == 0 && self.fine == RowInTile::Zero
    }

    pub fn tile_row(self, pixel_row: PixelRow) -> (TileRow, RowInTile) {
        let offset_pixel_row = self.to_u8().wrapping_add(pixel_row.to_u8());
        (
            TileRow::try_from_u8(offset_pixel_row / 8).unwrap(),
            RowInTile::from_u8(offset_pixel_row % 8).unwrap(),
        )
    }

    pub fn to_u8(self) -> u8 {
        (self.coarse.to_u8() << 3) | self.fine as u8
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
        assert_eq!(address.address, 0b0011_1111_0000_0000);
        assert_eq!(address.to_u16(), 0b0011_1111_0000_0000);
    }

    #[test]
    fn reduce_bit_1_from_y_scroll() {
        let mut address = PpuAddress::from_u16(0);
        address.set_y_scroll(0b1111_1111);
        assert_eq!(address.address, 0b0111_0011_1110_0000);
        assert_eq!(address.to_u16(), 0b0011_0011_1110_0000);
    }

    #[test]
    fn wrap_advance() {
        let mut address = PpuAddress::from_u16(0x3FFF);
        address.advance(1);
        assert_eq!(address.address, 0x0000);
        assert_eq!(address.to_u16(), 0x0000);
    }

    #[test]
    fn set_x_scroll() {
        let mut address = PpuAddress::from_u16(0);
        assert_eq!(address.x_scroll().to_u8(), 0x00);
        address.set_x_scroll(0xFF);
        assert_eq!(address.x_scroll().to_u8(), 0xFF);
    }

    #[test]
    fn set_y_scroll() {
        let mut address = PpuAddress::from_u16(0);
        assert_eq!(address.y_scroll().to_u8(), 0x00);
        address.set_y_scroll(0xFF);
        assert_eq!(address.y_scroll().to_u8(), 0xFF);
    }

    #[test]
    fn set_x_y_scroll() {
        let mut address = PpuAddress::from_u16(0);
        assert_eq!(address.x_scroll().to_u8(), 0x00);
        assert_eq!(address.y_scroll().to_u8(), 0x00);
        address.set_x_scroll(0xFD);
        address.set_y_scroll(0xFF);
        assert_eq!(address.x_scroll().to_u8(), 0xFD);
        assert_eq!(address.y_scroll().to_u8(), 0xFF);
    }
}
