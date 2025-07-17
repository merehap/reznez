use modular_bitfield::Specifier;

use crate::mapper::KIBIBYTE;

const PATTERN_TABLE_SIZE: u32 = 4 * KIBIBYTE;

#[derive(PartialEq, Eq, Clone, Copy, Debug, Specifier)]
pub enum PatternTableSide {
    Left,
    Right,
}

impl PatternTableSide {
    pub fn from_index(index: u32) -> PatternTableSide {
        assert!(index < 2 * PATTERN_TABLE_SIZE);
        if index / PATTERN_TABLE_SIZE == 0 {
            PatternTableSide::Left
        } else {
            PatternTableSide::Right
        }
    }

    pub fn to_start_end(self) -> (u32, u32) {
        match self {
            PatternTableSide::Left => (0x0000, PATTERN_TABLE_SIZE),
            PatternTableSide::Right => (PATTERN_TABLE_SIZE, 2 * PATTERN_TABLE_SIZE),
        }
    }
}

impl From<bool> for PatternTableSide {
    fn from(value: bool) -> PatternTableSide {
        if value {
            PatternTableSide::Right
        } else {
            PatternTableSide::Left
        }
    }
}

impl From<PatternTableSide> for u16 {
    fn from(value: PatternTableSide) -> Self {
        value as u16
    }
}