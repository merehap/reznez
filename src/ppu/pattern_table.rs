use std::fmt;

use crate::ppu::palette::palette_index::PaletteIndex;
use crate::util::get_bit;

pub struct PatternTable<'a>(&'a [u8; 0x2000]);

impl <'a> PatternTable<'a> {
    pub fn new(raw: &'a [u8; 0x2000]) -> PatternTable<'a> {
        PatternTable(raw)
    }

    /*
    pub fn tile_at(&self, side: PatternTableSide, tile_index: u8) -> Tile {
        let mut start_index = match side {
            PatternTableSide::Left => 0x0,
            PatternTableSide::Right => 0x1000,
        };

        start_index += 16 * (tile_index as usize);

        Tile::new((&self.0[start_index..start_index + 16]).try_into().unwrap())
    }
    */

    #[inline]
    pub fn tile_sliver_at(
        &'a self,
        side: PatternTableSide,
        tile_index: u8,
        row_in_tile: usize,
        ) -> [Option<PaletteIndex>; 8] {

        let mut index = side as usize;
        index += 16 * (tile_index as usize);
        let low_index = index + row_in_tile;
        let high_index = low_index + 8;

        let low_byte = self.0[low_index];
        let high_byte = self.0[high_index];

        let mut tile_sliver = [None; 8];
        for column_in_tile in 0..8 {
            let low_bit = get_bit(low_byte, column_in_tile);
            let high_bit = get_bit(high_byte, column_in_tile);
            let index = match (low_bit, high_bit) {
                (false, false) => None,
                (true , false) => Some(PaletteIndex::One),
                (false, true ) => Some(PaletteIndex::Two),
                (true , true ) => Some(PaletteIndex::Three),
            };

            tile_sliver[column_in_tile] = index;
        }

        tile_sliver
        /*
        [
            self.palette_index_at(0, row_in_tile),
            self.palette_index_at(1, row_in_tile),
            self.palette_index_at(2, row_in_tile),
            self.palette_index_at(3, row_in_tile),
            self.palette_index_at(4, row_in_tile),
            self.palette_index_at(5, row_in_tile),
            self.palette_index_at(6, row_in_tile),
            self.palette_index_at(7, row_in_tile),
        ]
        */
        //*self.tile_at(side, tile_index).sliver_at(row_in_tile)
    }
}

impl fmt::Display for PatternTable<'_> {
    fn fmt(&self, f: &'_ mut fmt::Formatter) -> fmt::Result {
        for row in 0..16 {
            for column in 0..16 {
                for side in [PatternTableSide::Left, PatternTableSide::Right] {
                    for row_in_tile in 0..8 {
                        let tile_index = 16 * row + column;
                        let sliver = self.tile_sliver_at(side, tile_index, row_in_tile);
                        for pixel in sliver {
                            let c = if let Some(pixel) = pixel {
                                char::from_digit(pixel as u32, 10).unwrap()
                            } else {
                                '-'
                            };

                            write!(f, "{}", c)?;
                        }

                        write!(f, " ")?;
                    }

                    write!(f, "  ")?;
                }

                writeln!(f)?;
            }

            writeln!(f)?;
        }

        Ok(())
    }
}

#[derive(Clone, Copy, Debug)]
pub enum PatternTableSide {
    Left  = 0x0000,
    Right = 0x1000,
}
