#[derive(Clone, Copy)]
pub struct TileNumber(u16);

const MAX_TILE_NUMBER: u16 = 959;

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

    pub fn to_usize(self) -> usize {
        self.0 as usize
    }
}
