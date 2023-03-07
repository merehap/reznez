#[rustfmt::skip]

use std::fmt;

use num_traits::FromPrimitive;

use crate::ppu::name_table::background_tile_index::{TileColumn, TileRow};
use crate::ppu::name_table::name_table_quadrant::NameTableQuadrant;
use crate::ppu::pattern_table::{PatternTableSide, PatternIndex};
use crate::ppu::pixel_index::{ColumnInTile, PixelColumn, PixelRow, RowInTile};
use crate::ppu::register::registers::ctrl::AddressIncrement;

/*
 * 0 123 45 6789A BCDEF
 * 0 yyy NN YYYYY XXXXX
 * | ||| || ||||| +++++-- Coarse X Scroll
 * | ||| || +++++-------- Coarse Y Scroll
 * | ||| ++-------------- Nametable Quadrant
 * | +++----------------- Fine Y Scroll
 * +--------------------- Unused, always zero
 */
#[derive(Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub struct PpuAddress {
    address: u16,
    fine_x_scroll: ColumnInTile,
}

impl PpuAddress {
    pub const ZERO: PpuAddress = PpuAddress {
        address: 0x0000,
        fine_x_scroll: ColumnInTile::Zero,
    };
    // 0x3F00
    pub const PALETTE_TABLE_START: PpuAddress = PpuAddress {
        address: 0x3F00,
        fine_x_scroll: ColumnInTile::Zero,
    };

    pub fn from_u16(value: u16) -> PpuAddress {
        PpuAddress {
            address: value & 0b0011_1111_1111_1111,
            fine_x_scroll: ColumnInTile::Zero,
        }
    }

    /*
     * 0123 45 6789A BCDEF
     * 0001 NN RRRRR CCCCC
     *      || ||||| +++++-- Tile Column
     *      || +++++-------- Tile Row
     *      ++-------------- Nametable Quadrant
     */
    pub fn in_name_table(
        quadrant: NameTableQuadrant,
        tile_column: TileColumn,
        tile_row: TileRow,
    ) -> PpuAddress {
        PpuAddress::from_u16(
            0x2000
            | (0x400 * quadrant as u16)
            | (TileColumn::COLUMN_COUNT as u16 * tile_row.to_u16())
            | tile_column.to_u16()
        )
    }

    /*
     * 0123 45 6789A BCDEF
     * 0010 NN RRRRR CCCCC
     *      || ||||| +++++-- Tile Column
     *      || +++++-------- Tile Row
     *      ++-------------- Nametable Quadrant
     */
    pub fn in_attribute_table(
        quadrant: NameTableQuadrant,
        tile_column: TileColumn,
        tile_row: TileRow,
    ) -> PpuAddress {
        PpuAddress::from_u16(
            0x23C0
            + 0x400 * quadrant as u16
            + (TileColumn::COLUMN_COUNT as u16) / 4 * (tile_row.to_u16() / 4)
            + tile_column.to_u16() / 4
        )
    }

    /*
     * 012 3 456789AB C DEF
     * 000 S PPPPPPPP H RRR
     *     | |||||||| | +++-- Row In Tile
     *     | |||||||| +------ Select High Byte (or Low Byte)
     *     | ++++++++-------- Pattern Index
     *     +----------------- Pattern Table Side
     */
    pub fn in_pattern_table(
        side: PatternTableSide,
        pattern_index: PatternIndex,
        row_in_tile: RowInTile,
        select_high_byte: bool,
    ) -> PpuAddress {
        PpuAddress::from_u16(
            (0x1000 * side as u16)
            | pattern_index.to_u16() << 4
            | if select_high_byte { 0x8 } else { 0x0 }
            | row_in_tile as u16
        )
    }

    pub fn advance(&mut self, address_increment: AddressIncrement) {
        if address_increment == AddressIncrement::Right {
            let mut coarse_x_scroll = self.coarse_x_scroll();
            let wrapped = coarse_x_scroll.increment();
            self.set_coarse_x_scroll(coarse_x_scroll);
            if !wrapped {
                return;
            }
        }

        let mut coarse_y_scroll = self.coarse_y_scroll();
        let wrapped = coarse_y_scroll.increment();
        self.set_coarse_y_scroll(coarse_y_scroll);
        if wrapped {
            let mut name_table_quadrant = self.name_table_quadrant();
            let wrapped = name_table_quadrant.increment();
            self.set_name_table_quadrant(name_table_quadrant);
            if wrapped {
                let mut fine_y_scroll = self.fine_y_scroll();
                fine_y_scroll.increment_low_bits();
                self.set_fine_y_scroll(fine_y_scroll);
            }
        }
    }

    pub fn increment_coarse_x_scroll(&mut self) {
        let mut coarse_x_scroll = self.coarse_x_scroll();
        let wrapped = coarse_x_scroll.increment();
        self.set_coarse_x_scroll(coarse_x_scroll);
        if wrapped {
            let mut name_table_quadrant = self.name_table_quadrant();
            name_table_quadrant = name_table_quadrant.next_horizontal();
            self.set_name_table_quadrant(name_table_quadrant);
        }
    }

    pub fn increment_fine_y_scroll(&mut self) {
        let mut fine_y_scroll = self.fine_y_scroll();
        let wrapped = fine_y_scroll.increment();
        self.set_fine_y_scroll(fine_y_scroll);
        if wrapped {
            let mut coarse_y_scroll = self.coarse_y_scroll();
            let wrapped = coarse_y_scroll.increment_visible();
            self.set_coarse_y_scroll(coarse_y_scroll);
            if wrapped {
                let mut name_table_quadrant = self.name_table_quadrant();
                name_table_quadrant = name_table_quadrant.next_vertical();
                self.set_name_table_quadrant(name_table_quadrant);
            }
        }
    }

    pub fn to_pending_data_source(self) -> PpuAddress {
        let mut data_source = self;
        if data_source.to_u16() >= 0x3000 {
            data_source = PpuAddress::from_u16(data_source.to_u16() - 0x1000);
        }

        data_source
    }

    pub fn name_table_quadrant(self) -> NameTableQuadrant {
        NameTableQuadrant::from_last_two_bits((self.address >> 10) as u8)
    }

    pub fn name_table_location(self) -> Option<(NameTableQuadrant, u16)> {
        if self.address >= 0x2000 && self.address < 0x3F00 {
            Some((
                    NameTableQuadrant::from_last_two_bits((self.address >> 10) as u8),
                    self.address & 0b11_1111_1111,
                ))
        } else {
            None
        }
    }

    pub fn is_in_attribute_table(self) -> bool {
        if self.address >= 0x2000 && self.address < 0x3F00 {
            self.address % 0x400 >= 0x3C0
        } else {
            false
        }
    }

    /*
     * 0123456789ABCDEF
     * -----------01234  $SCROLL#1
     * ----67----------  $CTRL
     */
    pub fn x_scroll(self) -> XScroll {
        XScroll {
            coarse: self.coarse_x_scroll(),
            fine: self.fine_x_scroll,
        }
    }

    fn coarse_x_scroll(self) -> TileColumn {
        TileColumn::try_from_u8(self.address as u8 & 0b11111).unwrap()
    }

    /*
     * 0123456789ABCDEF
     *  567--01234-----  $SCROLL#2
     *  ---67----------  $CTRL
     */
    pub fn y_scroll(self) -> YScroll {
        YScroll {
            coarse: self.coarse_y_scroll(),
            fine: self.fine_y_scroll(),
        }
    }

    fn coarse_y_scroll(self) -> TileRow {
        TileRow::try_from_u8(((self.address & 0b1111100000) >> 5) as u8).unwrap()
    }

    fn fine_y_scroll(self) -> RowInTile {
        RowInTile::from_u8(((self.address & 0b0111_0000_0000_0000) >> 12) as u8).unwrap()
    }

    pub fn set_name_table_quadrant(&mut self, quadrant: NameTableQuadrant) {
        self.address &= 0b0111_0011_1111_1111;
        self.address |= (quadrant as u16) << 10;
    }

    pub fn set_x_scroll(&mut self, value: u8) {
        let value = XScroll::from_u8(value);
        self.fine_x_scroll = value.fine();
        self.set_coarse_x_scroll(value.coarse());
    }

    fn set_coarse_x_scroll(&mut self, coarse_x: TileColumn) {
        self.address &= 0b1111_1111_1110_0000;
        self.address |= coarse_x.to_u16();
    }

    pub fn set_y_scroll(&mut self, value: u8) {
        let value = YScroll::from_u8(value);
        self.set_coarse_y_scroll(value.coarse());
        self.set_fine_y_scroll(value.fine());
    }

    fn set_coarse_y_scroll(&mut self, coarse_y: TileRow) {
        self.address &= 0b0111_1100_0001_1111;
        self.address |= coarse_y.to_u16() << 5;
    }

    fn set_fine_y_scroll(&mut self, fine_y: RowInTile) {
        self.address &= 0b0000_1111_1111_1111;
        self.address |= (fine_y as u16) << 12;
    }

    pub fn copy_x_scroll(&mut self, other: PpuAddress) {
        self.set_x_scroll(other.x_scroll().to_u8());
    }

    pub fn copy_y_scroll(&mut self, other: PpuAddress) {
        self.set_y_scroll(other.y_scroll().to_u8());
    }

    pub fn copy_name_table_quadrant(&mut self, other: PpuAddress) {
        self.set_name_table_quadrant(other.name_table_quadrant());
    }

    pub fn copy_horizontal_name_table_side(&mut self, other: PpuAddress) {
        let mut name_table_quadrant = self.name_table_quadrant();
        name_table_quadrant.copy_horizontal_side_from(other.name_table_quadrant());
        self.set_name_table_quadrant(name_table_quadrant);
    }

    pub fn set_high_byte(&mut self, value: u8) {
        // Lose the top bit of the fine y scroll.
        self.address &= 0b0000_0000_1111_1111;
        self.address |= u16::from(value & 0b0011_1111) << 8;
    }

    pub fn set_low_byte(&mut self, value: u8) {
        self.address &= 0b1111_1111_0000_0000;
        self.address |= u16::from(value);
    }

    pub fn to_u16(self) -> u16 {
        // Chop off the top bit of fine y to leave a 14-bit representation.
        self.to_scroll_u16() & 0b0011_1111_1111_1111
    }

    pub fn to_usize(self) -> usize {
        usize::from(self.to_u16())
    }

    pub fn to_scroll_u16(self) -> u16 {
        self.address
    }

    pub fn pattern_table_side(self) -> PatternTableSide {
        if self.address & 0b0001_0000_0000_0000 == 0 {
            PatternTableSide::Left
        } else {
            PatternTableSide::Right
        }
    }
}

impl PartialEq for PpuAddress {
    fn eq(&self, rhs: &PpuAddress) -> bool {
        self.to_u16() == rhs.to_u16()
    }
}

impl fmt::Display for PpuAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "${:04X}", self.to_u16())
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

    fn from_u8(value: u8) -> XScroll {
        XScroll {
            coarse: TileColumn::try_from_u8(value >> 3).unwrap(),
            fine: ColumnInTile::from_u8(value & 0b111).unwrap(),
        }
    }

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_and_to_u16() {
        for address in 0x0000..=0xFFFF {
            assert_eq!(address & 0x3FFF, PpuAddress::from_u16(address).to_u16());
        }
    }

    #[test]
    fn reduce_bit_1_from_high_byte() {
        let mut address = PpuAddress::from_u16(0);
        address.set_high_byte(0b1111_1111);
        assert_eq!(address.to_scroll_u16(), 0b0011_1111_0000_0000);
        assert_eq!(address.to_u16(), 0b0011_1111_0000_0000);
    }

    #[test]
    fn reduce_bit_1_from_y_scroll() {
        let mut address = PpuAddress::from_u16(0);
        address.set_y_scroll(0b1111_1111);
        assert_eq!(address.to_scroll_u16(), 0b0111_0011_1110_0000);
        assert_eq!(address.to_u16(), 0b0011_0011_1110_0000);
    }

    #[test]
    fn wrap_advance() {
        let mut address = PpuAddress::from_u16(0x3FFF);
        address.advance(AddressIncrement::Right);
        assert_eq!(address.to_scroll_u16(), 0x0000);
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
