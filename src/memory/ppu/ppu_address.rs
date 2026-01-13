use std::fmt;
use std::marker::ConstParamTy;

use num_traits::FromPrimitive;
use splitbits::{splitbits_named, splitbits_named_into_ux, splitbits_named_ux, combinebits, replacebits};

use crate::mapper::KIBIBYTE_U16;
use crate::ppu::name_table::background_tile_index::{TileColumn, TileRow};
use crate::ppu::name_table::name_table_quadrant::NameTableQuadrant;
use crate::ppu::pattern_table_side::PatternTableSide;
use crate::ppu::pixel_index::{ColumnInTile, PixelColumn, PixelRow, RowInTile};
use crate::ppu::register::ppu_registers::AddressIncrement;
use crate::ppu::tile_number::TileNumber;

/*
 * 0 123 45 6789A BCDEF
 * 0 yyy NN YYYYY XXXXX
 * | ||| || ||||| +++++-- Coarse X Scroll
 * | ||| || +++++-------- Coarse Y Scroll
 * | ||| ++-------------- Nametable Quadrant
 * | +++----------------- Fine Y Scroll
 * +--------------------- Unused, always zero
 */
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug, Default, ConstParamTy)]
pub struct PpuAddress {
    address: u16,
}

impl PpuAddress {
    pub const ZERO: PpuAddress = PpuAddress {
        address: 0x0000,
    };

    pub const fn from_u16(value: u16) -> PpuAddress {
        PpuAddress {
            address: value & 0b0011_1111_1111_1111,
        }
    }

    pub fn in_name_table(n: NameTableQuadrant, c: TileColumn, r: TileRow) -> PpuAddress {
        PpuAddress::from_u16(combinebits!("0010 nn rrrrr ccccc"))
    }

    pub fn in_attribute_table(n: NameTableQuadrant, c: TileColumn, r: TileRow) -> PpuAddress {
        let r = r.to_u16() / 4;
        let c = c.to_u16() / 4;
        PpuAddress::from_u16(combinebits!("0010 nn 1111r rrccc"))
    }

    pub fn in_pattern_table(s: PatternTableSide, p: TileNumber, r: RowInTile, select_high: bool) -> PpuAddress {
        let h = select_high;
        PpuAddress::from_u16(combinebits!("000s pppp pppp hrrr"))
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
        splitbits_named_ux!(self.to_u16(), ".... nn.. .... ....").into()
    }

    pub fn name_table_location(self) -> Option<(NameTableQuadrant, u32)> {
        if self.to_u16() >= 0x2000 && self.to_u16() < 0x3F00 {
            Some(splitbits_named_into_ux!(self.to_u16(), ".... nnll llll llll"))
        } else {
            None
        }
    }

    pub fn is_in_pattern_table(self) -> bool {
        self.to_u16() < 0x2000
    }

    pub fn is_in_name_table_proper(self) -> bool {
        if self.to_u16() >= 0x2000 && self.to_u16() < 0x3F00 {
            self.to_u16() % 0x400 < 0x3C0
        } else {
            false
        }
    }

    pub fn is_in_attribute_table(self) -> bool {
        if self.to_u16() >= 0x2000 && self.to_u16() < 0x3F00 {
            self.to_u16() % 0x400 >= 0x3C0
        } else {
            false
        }
    }

    pub fn is_in_palette_table(self) -> bool {
        matches!(self.to_u16(), 0x3F00..=0x3FFF)
    }

    pub fn coarse_x_scroll(self) -> TileColumn {
        splitbits_named_ux!(self.to_u16(), ".... .... ...x xxxx").into()
    }

    pub fn set_name_table_quadrant(&mut self, n: NameTableQuadrant) {
        self.address = replacebits!(self.address, "0... nn.. .... ....");
    }

    pub fn set_coarse_x_scroll(&mut self, x: TileColumn) {
        self.address = replacebits!(self.address, ".... .... ...x xxxx");
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

    pub fn set_y_scroll(&mut self, value: u8) {
        let value = YScroll::from_u8(value);
        self.set_coarse_y_scroll(value.coarse());
        self.set_fine_y_scroll(value.fine());
    }

    pub fn coarse_y_scroll(self) -> TileRow {
        splitbits_named_into_ux!(self.address, "...... yyyyy .....")
    }

    fn set_coarse_y_scroll(&mut self, y: TileRow) {
        self.address = replacebits!(self.address, "...... yyyyy .....");
    }

    pub fn fine_y_scroll(self) -> RowInTile {
        splitbits_named_into_ux!(self.address, ". yyy ............")
    }

    fn set_fine_y_scroll(&mut self, y: RowInTile) {
        self.address = replacebits!(self.address, ". yyy ............");
    }

    pub fn copy_y_scroll(&mut self, other: PpuAddress) {
        self.set_y_scroll(other.y_scroll().to_u8());
    }

    pub fn copy_name_table_quadrant(&mut self, other: PpuAddress) {
        self.set_name_table_quadrant(other.name_table_quadrant());
    }

    pub fn set_high_byte(&mut self, h: u8) {
        // Lose the top bit of the fine y scroll.
        let h = h & 0b0011_1111;
        self.address = replacebits!(self.address, "00hh hhhh .... ....");
    }

    pub fn set_low_byte(&mut self, l: u8) {
        self.address = replacebits!(self.address, ".... .... llll llll");
    }

    pub const fn to_u16(self) -> u16 {
        // Chop off the top bit of fine y to leave a 14-bit representation.
        self.address & 0b0011_1111_1111_1111
    }

    pub fn to_u32(self) -> u32 {
        u32::from(self.to_u16())
    }

    pub fn pattern_table_side(self) -> PatternTableSide {
        splitbits_named!(self.to_u16(), "...p .... .... ....").into()
    }

    #[inline]
    #[rustfmt::skip]
    pub fn to_name_table_index(self) -> (NameTableQuadrant, u16) {
        const NAME_TABLE_START:    u16 = 0x2000;
        const MIRROR_START:        u16 = 0x3000;
        const PALETTE_TABLE_START: u16 = 0x3F00;

        let address = self.to_u16();
        assert!(address >= NAME_TABLE_START);
        assert!(address < PALETTE_TABLE_START);

        let mut index = address;
        if index >= MIRROR_START {
            index -= 0x1000;
        }

        let index = index - NAME_TABLE_START;

        let name_table_quadrant = NameTableQuadrant::from_u16(index / KIBIBYTE_U16).unwrap();
        let index = index % KIBIBYTE_U16;
        (name_table_quadrant, index)
    }

    pub fn to_palette_ram_index(self) -> u32 {
        const PALETTE_TABLE_START: u32 = 0x3F00;
        const HIGH_ADDRESS_START: u32 = 0x4000;

        let mut address = self.to_u32();
        assert!(address >= PALETTE_TABLE_START);
        assert!(address < HIGH_ADDRESS_START);

        // Mirror address down.
        address %= 0x20;
        if matches!(address, 0x10 | 0x14 | 0x18 | 0x1C) {
            address -= 0x10;
        }

        address
    }
}

impl fmt::Display for PpuAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "${:04X}", self.to_u16())
    }
}

#[derive(Clone, Copy, Debug)]
pub struct XScroll {
    pub coarse: TileColumn,
    pub fine: ColumnInTile,
}

impl XScroll {
    pub const ZERO: XScroll =
        XScroll { coarse: TileColumn::ZERO, fine: ColumnInTile::Zero };

    pub fn from_u8(value: u8) -> XScroll {
        let (coarse, fine) = splitbits_named_into_ux!(value, "cccccfff");
        XScroll { coarse, fine }
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
        let (coarse, fine) = splitbits_named_ux!(offset_pixel_column, "cccccfff");
        (coarse.into(), fine.into())
    }

    pub fn to_u8(self) -> u8 {
        combinebits!(self.coarse, self.fine, "cccccfff")
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
        let (coarse, fine) = splitbits_named_into_ux!(value, "cccccfff");
        YScroll { coarse, fine }
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
        splitbits_named_into_ux!(offset_pixel_row, "cccccfff")
    }

    pub fn to_u8(self) -> u8 {
        combinebits!(self.coarse, self.fine, "cccccfff")
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
        address.advance(AddressIncrement::Right);
        assert_eq!(address.address, 0x0000);
        assert_eq!(address.to_u16(), 0x0000);
    }

    #[test]
    fn set_y_scroll() {
        let mut address = PpuAddress::from_u16(0);
        assert_eq!(address.y_scroll().to_u8(), 0x00);
        address.set_y_scroll(0xFF);
        assert_eq!(address.y_scroll().to_u8(), 0xFF);
    }
}