const COLUMN_COUNT: u16 = 32;
const ROW_COUNT: u16 = 30;
const MAX_TILE_NUMBER: u16 = COLUMN_COUNT * ROW_COUNT - 1;

#[derive(Clone, Copy)]
pub struct TileNumber(u16);

impl TileNumber {
    pub fn from_u16(number: u16) -> Result<TileNumber, String> {
        if number > MAX_TILE_NUMBER {
            return Err(format!(
                "Tile number must not be greater than {}.",
                MAX_TILE_NUMBER,
            ));
        }

        Ok(TileNumber(number))
    }

    pub fn iter() -> TileNumberIterator {
        TileNumberIterator {tile_number: TileNumber(0)}
    }

    pub fn to_usize(self) -> usize {
        self.0 as usize
    }

    pub fn column(self) -> u16 {
        self.0 % 32
    }

    pub fn row(self) -> u16 {
        self.0 / 32
    }
}

pub struct TileNumberIterator {
    tile_number: TileNumber,
}

impl Iterator for TileNumberIterator {
    type Item = TileNumber;

    fn next(&mut self) -> Option<TileNumber> {
        if self.tile_number.0 > MAX_TILE_NUMBER {
            return None;
        }

        let result = self.tile_number;
        self.tile_number.0 += 1;
        Some(result)
    }
}
