#[rustfmt::skip]

use std::fmt;

use num_traits::FromPrimitive;

use crate::ppu::name_table::background_tile_index::{TileColumn, TileRow};
use crate::ppu::name_table::name_table_quadrant::NameTableQuadrant;
use crate::ppu::pixel_index::{ColumnInTile, PixelColumn, PixelRow, RowInTile};
use crate::ppu::register::registers::ctrl::AddressIncrement;

const VERTICAL_NAME_TABLE_MASK: u16   = 0b0000_1000_0000_0000;
const HORIZONTAL_NAME_TABLE_MASK: u16 = 0b0000_0100_0000_0000;
const COARSE_Y_MASK: u16              = 0b0000_0011_1110_0000;
const COARSE_X_MASK: u16              = 0b0000_0000_0001_1111;

const NAME_TABLE_MASK: u16 = VERTICAL_NAME_TABLE_MASK | HORIZONTAL_NAME_TABLE_MASK;
const FINE_Y_ZERO_TOP_BIT_MASK: u16   = 0b0011_0000_0000_0000;

/*
 * 0 123 45 6789A BCDEF
 * . yyy NN YYYYY XXXXX
 * | ||| || ||||| +++++-- Coarse X Scroll
 * | ||| || +++++-------- Coarse Y Scroll
 * | ||| ++-------------- Nametable Select
 * | +++----------------- Fine Y Scroll
 * +--------------------- Unused
 */
#[derive(Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub struct PpuAddress {
    fine_y_scroll: RowInTile,
    name_table_quadrant: NameTableQuadrant,
    coarse_y_scroll: TileRow,
    coarse_x_scroll: TileColumn,

    fine_x_scroll: ColumnInTile,
}

impl PpuAddress {
    pub const ZERO: PpuAddress = PpuAddress {
        fine_y_scroll: RowInTile::Zero,
        name_table_quadrant: NameTableQuadrant::TopLeft,
        coarse_y_scroll: TileRow::ZERO,
        coarse_x_scroll: TileColumn::ZERO,
        fine_x_scroll: ColumnInTile::Zero,
    };
    pub const PALETTE_TABLE_START: PpuAddress = PpuAddress {
        fine_y_scroll: RowInTile::Three,
        name_table_quadrant: NameTableQuadrant::BottomRight,
        coarse_y_scroll: TileRow::try_from_u8(7).unwrap(),
        coarse_x_scroll: TileColumn::ZERO,
        fine_x_scroll: ColumnInTile::Zero,
    };

    pub fn from_u16(value: u16) -> PpuAddress {
        PpuAddress {
            fine_y_scroll: RowInTile::from_u8(((value & FINE_Y_ZERO_TOP_BIT_MASK) >> 12) as u8).unwrap(),
            name_table_quadrant: FromPrimitive::from_u16((value & NAME_TABLE_MASK) >> 10).unwrap(),
            coarse_y_scroll: TileRow::try_from_u8(((value & COARSE_Y_MASK) >> 5) as u8).unwrap(),
            coarse_x_scroll: TileColumn::try_from_u8((value & COARSE_X_MASK) as u8).unwrap(),

            fine_x_scroll: ColumnInTile::Zero,
        }
    }

    pub fn advance(&mut self, address_increment: AddressIncrement) {
        if address_increment == AddressIncrement::Right {
            let wrapped = self.coarse_x_scroll.increment();
            if !wrapped {
                return;
            }
        }

        let wrapped = self.coarse_y_scroll.increment();
        if wrapped {
            let wrapped = self.name_table_quadrant.increment();
            if wrapped {
                self.fine_y_scroll.increment_low_bits();
            }
        }
    }

    pub fn increment_coarse_x_scroll(&mut self) {
        let wrapped = self.coarse_x_scroll.increment();
        if wrapped {
            self.name_table_quadrant = self.name_table_quadrant.next_horizontal();
        }
    }

    pub fn increment_fine_y_scroll(&mut self) {
        let wrapped = self.fine_y_scroll.increment();
        if wrapped {
            let wrapped = self.coarse_y_scroll.increment_visible();
            if wrapped {
                self.name_table_quadrant = self.name_table_quadrant.next_vertical();
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
        self.name_table_quadrant
    }

    /*
     * 0123456789ABCDEF
     * -----------01234  $SCROLL#1
     * ----67----------  $CTRL
     */
    pub fn x_scroll(self) -> XScroll {
        XScroll {
            coarse: self.coarse_x_scroll,
            fine: self.fine_x_scroll,
        }
    }

    /*
     * 0123456789ABCDEF
     *  567--01234-----  $SCROLL#2
     *  ---67----------  $CTRL
     */
    pub fn y_scroll(self) -> YScroll {
        YScroll {
            coarse: self.coarse_y_scroll,
            fine: self.fine_y_scroll,
        }
    }

    pub fn set_name_table_quadrant(&mut self, quadrant: NameTableQuadrant) {
        self.name_table_quadrant = quadrant;
    }

    pub fn set_x_scroll(&mut self, value: u8) {
        let value = XScroll::from_u8(value);
        self.coarse_x_scroll = value.coarse();
        self.fine_x_scroll = value.fine();
    }

    pub fn set_y_scroll(&mut self, value: u8) {
        let value = YScroll::from_u8(value);
        self.coarse_y_scroll = value.coarse();
        self.fine_y_scroll = value.fine();
    }

    pub fn copy_x_scroll(&mut self, other: PpuAddress) {
        self.set_x_scroll(other.x_scroll().to_u8());
    }

    pub fn copy_y_scroll(&mut self, other: PpuAddress) {
        self.set_y_scroll(other.y_scroll().to_u8());
    }

    pub fn copy_name_table_quadrant(&mut self, other: PpuAddress) {
        self.name_table_quadrant = other.name_table_quadrant;
    }

    pub fn copy_horizontal_name_table_side(&mut self, other: PpuAddress) {
        self.name_table_quadrant.copy_horizontal_side_from(other.name_table_quadrant);
    }

    pub fn set_high_byte(&mut self, value: u8) {
        self.fine_y_scroll = RowInTile::from_u8((value & 0b0011_0000) >> 4).unwrap();
        self.name_table_quadrant = FromPrimitive::from_u8((value & 0b0000_1100) >> 2).unwrap();
        self.coarse_y_scroll = TileRow::try_from_u8(((self.coarse_y_scroll.to_u8()) & !0b1_1000) | ((value & 0b11) << 3)).unwrap();
    }

    pub fn set_low_byte(&mut self, value: u8) {
        self.coarse_y_scroll = TileRow::try_from_u8(((self.coarse_y_scroll.to_u8()) & !0b111) | ((value & 0b1110_0000) >> 5)).unwrap();
        self.coarse_x_scroll = TileColumn::try_from_u8(value & 0b0001_1111).unwrap();
    }

    pub fn to_u16(self) -> u16 {
        // Chop off the top bit of fine y to leave a 14-bit representation.
        self.to_scroll_u16() & 0b0011_1111_1111_1111
    }

    pub fn to_usize(self) -> usize {
        usize::from(self.to_u16())
    }

    fn to_scroll_u16(self) -> u16 {
        let fine_y = (self.fine_y_scroll as u16) << 12;
        let quadrant = (self.name_table_quadrant as u16) << 10;
        let coarse_y = (self.coarse_y_scroll.to_u16()) << 5;
        let coarse_x = self.coarse_x_scroll.to_u16();
        fine_y | quadrant | coarse_y | coarse_x
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
        println!("Zero: {:016b}", address.to_u16());
        address.set_high_byte(0b1111_1111);
        println!("After: {:016b}", address.to_u16());
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
