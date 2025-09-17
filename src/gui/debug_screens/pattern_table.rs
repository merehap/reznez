// modular_bitfield pedantic clippy warnings
#![allow(clippy::cast_lossless, clippy::no_effect_underscore_binding, clippy::map_unwrap_or)]

use enum_iterator::all;

use crate::mapper::PatternTableSide;
use crate::memory::memory::Memory;
use crate::memory::raw_memory::RawMemorySlice;
use crate::ppu::palette::palette::Palette;
use crate::ppu::palette::rgbt::Rgbt;
use crate::ppu::pixel_index::{ColumnInTile, RowInTile};
use crate::ppu::tile_number::TileNumber;
use crate::util::bit_util::get_bit;
use crate::util::unit::KIBIBYTE;

const PATTERN_SIZE: u32 = 16;

// Used for debug window purposes only. The actual rendering pipeline deals with unabstracted bytes.
pub struct PatternTable<'a>([RawMemorySlice<'a>; 4]);

impl<'a> PatternTable<'a> {
    pub fn new(raw: [RawMemorySlice<'a>; 4]) -> PatternTable<'a> {
        PatternTable(raw)
    }

    pub fn from_mem(mem: &'a Memory, side: PatternTableSide) -> PatternTable<'a> {
        let ciram = mem.ciram();
        let chr_memory = mem.chr_memory();
        match side {
            PatternTableSide::Left => PatternTable::new(chr_memory.left_chunks(ciram)),
            PatternTableSide::Right => PatternTable::new(chr_memory.right_chunks(ciram)),
        }
    }

    #[inline]
    pub fn background_side(mem: &'a Memory) -> PatternTable<'a> {
        Self::from_mem(mem, mem.ppu_regs.background_table_side())
    }

    #[inline]
    pub fn sprite_side(mem: &'a Memory) -> PatternTable<'a> {
        Self::from_mem(mem, mem.ppu_regs.sprite_table_side())
    }


    pub fn render_pixel(
        &self,
        tile_number: TileNumber,
        column_in_tile: ColumnInTile,
        row_in_tile: RowInTile,
        palette: Palette,
        pixel: &mut Rgbt,
    ) {
        let index = PATTERN_SIZE * u32::from(tile_number);
        let low_index = index + row_in_tile as u32;
        let high_index = low_index + 8;

        let low_byte = self.read(low_index);
        let high_byte = self.read(high_index);

        let low_bit = get_bit(low_byte, column_in_tile as u32);
        let high_bit = get_bit(high_byte, column_in_tile as u32);
        *pixel = palette.rgbt_from_low_high(low_bit, high_bit);
    }

    fn read(&self, index: u32) -> u8 {
        let quadrant = index / KIBIBYTE;
        assert!(quadrant < 5);

        let offset = index % KIBIBYTE;

        self.0[quadrant as usize][offset]
    }

    pub fn render_background_tile(
        &self,
        tile_number: TileNumber,
        palette: Palette,
        tile: &mut Tile,
    ) {
        for row_in_tile in all::<RowInTile>() {
            self.render_pixel_sliver(
                tile_number,
                row_in_tile,
                palette,
                &mut tile.0[row_in_tile as usize],
            );
        }
    }

    #[inline]
    #[rustfmt::skip]
    pub fn render_pixel_sliver(
        &self,
        tile_number: TileNumber,
        row_in_tile: RowInTile,
        palette: Palette,
        tile_sliver: &mut [Rgbt; 8],
    ) {
        let index = PATTERN_SIZE * u32::from(tile_number);
        let low_index = index + row_in_tile as u32;
        let high_index = low_index + 8;

        let low_byte = self.read(low_index);
        let high_byte = self.read(high_index);

        for (column_in_tile, pixel) in tile_sliver.iter_mut().enumerate() {
            let low_bit = get_bit(low_byte, column_in_tile as u32);
            let high_bit = get_bit(high_byte, column_in_tile as u32);
            *pixel = palette.rgbt_from_low_high(low_bit, high_bit);
        }
    }
}

pub struct Tile(pub [[Rgbt; 8]; 8]);

impl Tile {
    pub fn new() -> Tile {
        Tile([[Rgbt::Transparent; 8]; 8])
    }

    pub fn row_mut(&mut self, row: RowInTile) -> &mut [Rgbt; 8] {
        &mut self.0[row as usize]
    }
}
