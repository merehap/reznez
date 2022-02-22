use enum_iterator::IntoEnumIterator;

pub struct PixelRow(u8);

impl PixelRow {
    const MAX: u8 = 239;

    pub fn try_from_u8(pixel_row: u8) -> Option<PixelRow> {
        if pixel_row <= PixelRow::MAX {
            Some(PixelRow(pixel_row))
        } else {
            None
        }
    }

    pub fn add_row_in_tile(self, row_in_tile: RowInTile) -> Option<PixelRow> {
        let value = self.0.checked_add(row_in_tile as u8)?;
        PixelRow::try_from_u8(value)
    }

    pub fn offset(self, offset: i16) -> PixelRow {
        // TODO: Is this a problem for Super Mario Bros?
        PixelRow::try_from_u8(((self.0 as i16 + offset).rem_euclid(256) as u8) % 240)
            .unwrap()
    }

    pub fn to_u8(self) -> u8 {
        self.0
    }
}

#[derive(Clone, Copy, IntoEnumIterator)]
pub enum RowInTile {
    Zero,
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
}
