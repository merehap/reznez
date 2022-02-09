use crate::ppu::palette::palette::Palette;
use crate::ppu::palette::palette_index::PaletteIndex;
use crate::ppu::palette::rgbt::Rgbt;
use crate::ppu::render::frame::Frame;
use crate::ppu::sprite::Sprite;
use crate::util::bit_util::get_bit;
use crate::util::mapped_array::MappedArray;

const PATTERN_TABLE_SIZE: usize = 0x1000;
const PATTERN_SIZE: usize = 16;

pub struct PatternTable<'a>(&'a MappedArray<4>);

impl <'a> PatternTable<'a> {
    pub fn new(raw: &MappedArray<4>) -> PatternTable {
        PatternTable(raw)
    }

    #[inline]
    pub fn render_tile_sliver(
        &self,
        pattern_index: PatternIndex,
        row_in_tile: usize,
        palette: Palette,
        tile_sliver: &mut [Rgbt; 8],
    ) {
        let index = PATTERN_SIZE * pattern_index.to_usize();
        let low_index = index + row_in_tile;
        let high_index = low_index + 8;

        let low_byte = self.0.read(low_index);
        let high_byte = self.0.read(high_index);

        for (column_in_tile, pixel) in tile_sliver.iter_mut().enumerate() {
            let low_bit = get_bit(low_byte, column_in_tile);
            let high_bit = get_bit(high_byte, column_in_tile);
            *pixel =
                match (low_bit, high_bit) {
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
    pub fn render_sprite_sliver(
        &self,
        sprite: Sprite,
        pattern_index: PatternIndex,
        is_sprite_0: bool,
        palette: Palette,
        frame: &mut Frame,
        column: u8,
        row: u8,
        row_in_sprite: usize,
    ) {
        frame.set_tile_sliver_priority(column, row, sprite.priority());

        let index = 16 * pattern_index.to_usize();
        let low_index = index + row_in_sprite;
        let high_index = low_index + 8;

        let low_byte = self.0.read(low_index);
        let high_byte = self.0.read(high_index);

        let flip = sprite.flip_horizontally();
        for column_in_sprite in 0..8 {
            let low_bit = get_bit(low_byte, column_in_sprite);
            let high_bit = get_bit(high_byte, column_in_sprite);
            let rgbt = match (low_bit, high_bit) {
                (false, false) => Rgbt::Transparent,
                (true , false) => Rgbt::Opaque(palette[PaletteIndex::One]),
                (false, true ) => Rgbt::Opaque(palette[PaletteIndex::Two]),
                (true , true ) => Rgbt::Opaque(palette[PaletteIndex::Three]),
            };
            let column_in_sprite =
                if flip {
                    7 - column_in_sprite
                } else {
                    column_in_sprite
                };
            if column as usize + column_in_sprite as usize >= Frame::WIDTH {
                break;
            }

            frame.set_sprite_pixel(column, row, column_in_sprite, rgbt, is_sprite_0);
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
    Left,
    Right,
}

impl PatternTableSide {
    pub fn from_index(index: usize) -> PatternTableSide {
        assert!(index < 2 * PATTERN_TABLE_SIZE);
        if index / PATTERN_TABLE_SIZE == 0 {
            PatternTableSide::Left
        } else {
            PatternTableSide::Right
        }
    }

    pub fn to_start_end(self) -> (usize, usize) {
        match self {
            PatternTableSide::Left  => (0x0000, PATTERN_TABLE_SIZE),
            PatternTableSide::Right => (PATTERN_TABLE_SIZE, 2 * PATTERN_TABLE_SIZE),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct PatternIndex(u8);

impl PatternIndex {
    pub fn new(value: u8) -> PatternIndex {
        PatternIndex(value)
    }
    pub fn to_tall_indexes(self) -> (PatternIndex, PatternIndex) {
        let first  = self.0 & 0b1111_1110;
        let second = self.0 | 0b0000_0001;
        (PatternIndex(first), PatternIndex(second))
    }
    pub fn into_wide_index(mut self) -> PatternIndex {
        self.0 &= 0b1111_1110;
        self
    }

    pub fn to_u8(self) -> u8 {
        self.0
    }

    pub fn to_usize(self) -> usize {
        usize::from(self.0)
    }
}
