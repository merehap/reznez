/*
use crate::ppu::name_table::name_table::NameTable;
use crate::ppu::name_table::background_tile_index::BackgroundTileIndex;
use crate::ppu::palette::palette_table_index::PaletteTableIndex;
use crate::ppu::pattern_table::PatternIndex;

pub struct ViewPort<'a> {
    base_name_table: NameTable<'a>,
    scroll_info: Option<ScrollInfo<'a>>,
}

impl <'a> ViewPort<'a> {
    #[inline]
    pub fn base_name_table_only(base_name_table: NameTable<'a>) -> ViewPort<'a> {
        ViewPort {
            base_name_table,
            scroll_info: None,
        }
    }

    #[inline]
    pub fn horizontal(
        base_name_table: NameTable<'a>,
        right_name_table: NameTable<'a>,
        scroll_offset: u8,
    ) -> ViewPort<'a> {

        let scroll_info = ScrollInfo {
            other_name_table: right_name_table,
            direction: Direction::Right,
            offset: scroll_offset,
        };

        ViewPort {
            base_name_table,
            scroll_info: Some(scroll_info),
        }
    }

    #[inline]
    pub fn vertical(
        base_name_table: NameTable<'a>,
        bottom_name_table: NameTable<'a>,
        scroll_offset: u8,
    ) -> ViewPort<'a> {

        let scroll_info = ScrollInfo {
            other_name_table: bottom_name_table,
            direction: Direction::Down,
            offset: scroll_offset,
        };

        ViewPort {
            base_name_table,
            scroll_info: Some(scroll_info),
        }
    }

    #[inline]
    pub fn tile_entry_at(
        &self,
        background_tile_index: BackgroundTileIndex,
    ) -> (PatternIndex, PaletteTableIndex) {

        self.base_name_table.tile_entry_at(background_tile_index)
            /*
        let column = background_tile_index.column() + 16;
        let row = background_tile_index.row();
        if let Ok(index) = BackgroundTileIndex::from_column_row(column, row) {
            self.base_name_table.tile_entry_at(index)
        } else {
            self.base_name_table.tile_entry_at(BackgroundTileIndex::from_column_row(0, 0).unwrap())
        }
        */

        /*
        if let Some(ScrollInfo {other_name_table, direction, offset}) = &self.scroll_info {
            use Direction::{Right, Down};
            let (name_table, column, row) =
                match direction {
                    Right if 8 * column >= *offset => {
                        (&self.base_name_table, column + offset / 8, row)
                    },
                    Right => {
                        (other_name_table     , column - (255 - offset / 8), row)
                    },
                    Down if row >= *offset =>
                        (&self.base_name_table, column, row + offset),
                    Down =>
                        (other_name_table     , column, offset - row),
                };
            let index = BackgroundTileIndex::from_column_row(column, row).unwrap();
            println!("Column, Row: {}, {}", column, row);
            name_table.tile_entry_at(index)
        } else {
            self.base_name_table.tile_entry_at(background_tile_index)
        }
        */
    }
}

pub struct ScrollInfo<'a> {
    other_name_table: NameTable<'a>,
    direction: Direction,
    offset: u8,
}

pub enum Direction {
    Right,
    Down,
}
*/
