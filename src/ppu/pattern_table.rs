//use std::fmt;

use crate::ppu::palette::palette::Palette;
use crate::ppu::palette::palette_index::PaletteIndex;
use crate::ppu::palette::rgbt::Rgbt;
use crate::ppu::render::frame::Frame;
use crate::ppu::sprite::Sprite;
use crate::util::bit_util::get_bit;

pub struct PatternTable<'a>(&'a [u8; 0x2000]);

impl <'a> PatternTable<'a> {
    pub fn new(raw: &'a [u8; 0x2000]) -> PatternTable<'a> {
        PatternTable(raw)
    }

    #[inline]
    pub fn render_background_tile_sliver(
        &'a self,
        side: PatternTableSide,
        pattern_index: PatternIndex,
        row_in_tile: usize,
        palette: Palette,
        tile_sliver: &mut [Rgbt; 8],
    ) {
        let index = side as usize + 16 * pattern_index.to_usize();
        let low_index = index + row_in_tile;
        let high_index = low_index + 8;

        let low_byte = self.0[low_index];
        let high_byte = self.0[high_index];

        for (column_in_tile, rgbt) in &mut tile_sliver.iter_mut().enumerate() {
            let low_bit = get_bit(low_byte, column_in_tile);
            let high_bit = get_bit(high_byte, column_in_tile);
            *rgbt = match (low_bit, high_bit) {
                (false, false) => Rgbt::Transparent,
                (true , false) => Rgbt::Opaque(palette[PaletteIndex::One]),
                (false, true ) => Rgbt::Opaque(palette[PaletteIndex::Two]),
                (true , true ) => Rgbt::Opaque(palette[PaletteIndex::Three]),
            };
        }
    }

    // No obvious way to reduce the number of parameters.
    #[allow(clippy::too_many_arguments)]
    #[inline]
    pub fn render_sprite_tile_sliver(
        &self,
        side: PatternTableSide,
        sprite: Sprite,
        palette: Palette,
        frame: &mut Frame,
        column: u8,
        row: u8,
        row_in_tile: usize,
    ) {
        frame.set_tile_sliver_priority(column, row, sprite.priority());

        let pattern_index = sprite.pattern_index();
        let index = side as usize + 16 * pattern_index.to_usize();
        let low_index = index + row_in_tile;
        let high_index = low_index + 8;

        let low_byte = self.0[low_index];
        let high_byte = self.0[high_index];

        let flip = sprite.flip_horizontally();
        for column_in_tile in 0..8 {
            let low_bit = get_bit(low_byte, column_in_tile);
            let high_bit = get_bit(high_byte, column_in_tile);
            let rgbt = match (low_bit, high_bit) {
                (false, false) => Rgbt::Transparent,
                (true , false) => Rgbt::Opaque(palette[PaletteIndex::One]),
                (false, true ) => Rgbt::Opaque(palette[PaletteIndex::Two]),
                (true , true ) => Rgbt::Opaque(palette[PaletteIndex::Three]),
            };
            let column_in_tile =
                if flip {
                    7 - column_in_tile
                } else {
                    column_in_tile
                };
            frame.set_sprite_pixel(column, row, column_in_tile, rgbt);
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

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum PatternTableSide {
    Left  = 0x0000,
    Right = 0x1000,
}

#[derive(Clone, Copy, Debug)]
pub struct PatternIndex(u8);

impl PatternIndex {
    pub fn new(value: u8) -> PatternIndex {
        PatternIndex(value)
    }

    pub fn to_usize(self) -> usize {
        self.0 as usize
    }
}
