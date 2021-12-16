//use std::fmt;

use crate::ppu::palette::palette::Palette;
use crate::ppu::palette::palette_index::PaletteIndex;
use crate::ppu::palette::rgb::Rgb;
use crate::util::get_bit;

pub struct PatternTable<'a>(&'a [u8; 0x2000]);

impl <'a> PatternTable<'a> {
    pub fn new(raw: &'a [u8; 0x2000]) -> PatternTable<'a> {
        PatternTable(raw)
    }

    #[inline]
    pub fn render_tile_sliver(
        &'a self,
        side: PatternTableSide,
        tile_index: u8,
        row_in_tile: usize,
        palette: Palette,
        universal_background_rgb: Rgb,
        tile_sliver: &mut [Rgb; 8],
        ) {

        let index = side as usize + 16 * (tile_index as usize);
        let low_index = index + row_in_tile;
        let high_index = low_index + 8;

        let low_byte = self.0[low_index];
        let high_byte = self.0[high_index];

        //let pixel_row = 8 * tile_number.row() + row_in_tile as u8;

        for (column_in_tile, rgb) in &mut tile_sliver.iter_mut().enumerate() {
            let low_bit = get_bit(low_byte, column_in_tile);
            let high_bit = get_bit(high_byte, column_in_tile);
            *rgb = match (low_bit, high_bit) {
                (false, false) => universal_background_rgb,
                (true , false) => palette[PaletteIndex::One],
                (false, true ) => palette[PaletteIndex::Two],
                (true , true ) => palette[PaletteIndex::Three],
            };

            /*
            let pixel_column =
                8 * tile_number.column() + column_in_tile as u8;
            */
        }
    }
}

/*
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
*/

#[derive(Clone, Copy, Debug)]
pub enum PatternTableSide {
    Left  = 0x0000,
    Right = 0x1000,
}
