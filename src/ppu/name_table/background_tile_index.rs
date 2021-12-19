const COLUMN_COUNT: u16 = 32;
const ROW_COUNT: u16 = 30;
const MAX_INDEX: u16 = COLUMN_COUNT * ROW_COUNT - 1;

#[derive(Clone, Copy)]
pub struct BackgroundTileIndex(u16);

impl BackgroundTileIndex {
    pub fn from_u16(number: u16) -> Result<BackgroundTileIndex, String> {
        if number > MAX_INDEX {
            return Err(format!(
                "Background tile index must not be greater than {}.",
                MAX_INDEX,
            ));
        }

        Ok(BackgroundTileIndex(number))
    }

    pub fn iter() -> BackgroundTileIndexIterator {
        BackgroundTileIndexIterator {index: BackgroundTileIndex(0)}
    }

    pub fn to_usize(self) -> usize {
        self.0 as usize
    }

    #[inline]
    pub fn column(self) -> u8 {
        (self.0 % 32).try_into().unwrap()
    }

    #[inline]
    pub fn row(self) -> u8 {
        (self.0 / 32).try_into().unwrap()
    }
}

pub struct BackgroundTileIndexIterator {
    index: BackgroundTileIndex,
}

impl Iterator for BackgroundTileIndexIterator {
    type Item = BackgroundTileIndex;

    fn next(&mut self) -> Option<BackgroundTileIndex> {
        if self.index.0 > MAX_INDEX {
            return None;
        }

        let result = self.index;
        self.index.0 += 1;
        Some(result)
    }
}
