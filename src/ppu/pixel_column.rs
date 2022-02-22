use enum_iterator::IntoEnumIterator;

#[derive(Clone, Copy)]
pub struct PixelColumn(u8);

impl PixelColumn {
    pub fn new(pixel_column: u8) -> PixelColumn {
        PixelColumn(pixel_column)
    }

    pub fn add_column_in_tile(self, column_in_tile: ColumnInTile) -> Option<PixelColumn> {
        let value = self.0.checked_add(column_in_tile as u8)?;
        Some(PixelColumn::new(value))
    }

    pub fn offset(self, offset: i16) -> Option<PixelColumn> {
        (self.0 as i16 + offset)
            .try_into()
            .ok()
            .map(|pc| PixelColumn(pc))
    }

    pub fn to_usize(self) -> usize {
        self.0 as usize
    }

}

#[derive(Clone, Copy, IntoEnumIterator)]
pub enum ColumnInTile {
    Zero,
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
}
