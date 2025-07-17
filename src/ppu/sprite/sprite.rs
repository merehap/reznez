use enum_iterator::all;
use num_traits::FromPrimitive;

use crate::ppu::palette::palette_table::PaletteTable;
use crate::ppu::palette::rgbt::Rgbt;
use crate::ppu::pattern_table::{TileNumber, PatternTable, Tile};
use crate::ppu::pixel_index::{ColumnInTile, PixelColumn, PixelRow, RowInTile};
use crate::ppu::render::frame::Frame;
use crate::ppu::sprite::sprite_attributes::{SpriteAttributes, Priority};
use crate::ppu::sprite::sprite_half::SpriteHalf;
use crate::ppu::sprite::sprite_height::SpriteHeight;
use crate::ppu::sprite::sprite_y::SpriteY;

/**
 * FOR DEBUG WINDOWS ONLY. Sprites never actual exist in this form during PPU rendering.
 */
#[derive(Clone, Copy, Debug)]
pub struct Sprite {
    x_coordinate: PixelColumn,
    y_coordinate: SpriteY,
    tile_number: TileNumber,
    attributes: SpriteAttributes,
}

impl Sprite {
    #[inline]
    pub fn from_u32(value: u32) -> Sprite {
        let [y_coordinate, raw_tile_number, attributes, x_coordinate] =
            value.to_be_bytes();

        Sprite {
            x_coordinate: PixelColumn::new(x_coordinate),
            y_coordinate: SpriteY::new(y_coordinate),
            tile_number: TileNumber::new(raw_tile_number),
            attributes: SpriteAttributes::from_u8(attributes),
        }
    }

    pub fn priority(self) -> Priority {
        self.attributes.priority()
    }

    pub fn tile_number(self) -> TileNumber {
        self.tile_number
    }

    // For debug screens only.
    pub fn render_normal_height(
        self,
        pattern_table: &PatternTable,
        palette_table: &PaletteTable,
    ) -> Tile {
        self.render_tile(
            SpriteHeight::Normal,
            SpriteHalf::Top,
            pattern_table,
            palette_table,
        )
    }

    // For debug screens only.
    pub fn render_tall(
        self,
        pattern_table: &PatternTable,
        palette_table: &PaletteTable,
    ) -> (Tile, Tile) {
        use SpriteHalf::*;
        use SpriteHeight::Tall;
        (
            self.render_tile(Tall, Top, pattern_table, palette_table),
            self.render_tile(Tall, Bottom, pattern_table, palette_table),
        )
    }

    pub fn render_sliver(
        self,
        row: PixelRow,
        sprite_height: SpriteHeight,
        pattern_table: &PatternTable,
        palette_table: &PaletteTable,
        is_sprite_0: bool,
        frame: &mut Frame,
    ) {
        let Some((sprite_half, row_in_half, _visible)) =
            self.y_coordinate.row_in_sprite(self.attributes.flip_vertically(), sprite_height, row) else {

            return;
        };

        let mut sprite_sliver = [Rgbt::Transparent; 8];
        self.render_sliver_from_sprite_half(
            sprite_height,
            sprite_half,
            row_in_half,
            pattern_table,
            palette_table,
            &mut sprite_sliver,
        );

        for (column_in_sprite, &pixel) in sprite_sliver.iter().enumerate() {
            let column_in_sprite = ColumnInTile::from_usize(column_in_sprite).unwrap();
            if let Rgbt::Opaque(_) = pixel && let Some(column) = self.x_coordinate.add_column_in_tile(column_in_sprite) {
                frame.set_sprite_pixel(
                    column,
                    row,
                    pixel,
                    self.priority(),
                    is_sprite_0,
                );
            }
        }
    }

    fn render_tile(
        self,
        sprite_height: SpriteHeight,
        sprite_half: SpriteHalf,
        pattern_table: &PatternTable,
        palette_table: &PaletteTable,
    ) -> Tile {
        let mut tile = Tile::new();
        for row in all::<RowInTile>() {
            self.render_sliver_from_sprite_half(
                sprite_height,
                sprite_half,
                row,
                pattern_table,
                palette_table,
                tile.row_mut(row),
            );
        }

        tile
    }

    fn render_sliver_from_sprite_half(
        self,
        sprite_height: SpriteHeight,
        sprite_half: SpriteHalf,
        mut row_in_half: RowInTile,
        pattern_table: &PatternTable,
        palette_table: &PaletteTable,
        sprite_sliver: &mut [Rgbt; 8],
    ) {
        #[rustfmt::skip]
        let tile_number = match (sprite_height, sprite_half) {
            (SpriteHeight::Normal, SpriteHalf::Top) => self.tile_number,
            (SpriteHeight::Normal, SpriteHalf::Bottom) => unreachable!(),
            (SpriteHeight::Tall,   SpriteHalf::Top) => self.tile_number.to_tall_tile_numbers().0,
            (SpriteHeight::Tall,   SpriteHalf::Bottom) => self.tile_number.to_tall_tile_numbers().1,
        };

        if self.attributes.flip_vertically() {
            row_in_half = row_in_half.flip();
        }

        let sprite_palette = palette_table.sprite_palette(self.attributes.palette_table_index());
        pattern_table.render_pixel_sliver(
            tile_number,
            row_in_half,
            sprite_palette,
            sprite_sliver,
        );

        if self.attributes.flip_horizontally() {
            for i in 0..sprite_sliver.len() / 2 {
                sprite_sliver.swap(i, 7 - i);
            }
        }
    }
}
