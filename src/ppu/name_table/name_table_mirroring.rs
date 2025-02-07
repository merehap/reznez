use std::fmt;

use crate::memory::ppu::vram::VramSide;
use crate::ppu::name_table::name_table_quadrant::NameTableQuadrant;

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct NameTableMirroring {
    // TopLeft, TopRight, BottomLeft, BottomRight
    quadrants: [VramSide; 4],
}

impl NameTableMirroring {
    pub const VERTICAL: NameTableMirroring =
        build([VramSide::Left, VramSide::Right, VramSide::Left, VramSide::Right]);
    pub const HORIZONTAL: NameTableMirroring =
        build([VramSide::Left, VramSide::Left, VramSide::Right, VramSide::Right]);
    pub const ONE_SCREEN_LEFT_BANK: NameTableMirroring =
        build([VramSide::Left, VramSide::Left, VramSide::Left, VramSide::Left]);
    pub const ONE_SCREEN_RIGHT_BANK: NameTableMirroring =
        build([VramSide::Right, VramSide::Right, VramSide::Right, VramSide::Right]);

    pub fn vram_side_at_quadrant(self, quadrant: NameTableQuadrant) -> VramSide {
        self.quadrants[quadrant as usize]
    }

    pub fn is_vertical(self) -> bool {
        self == NameTableMirroring::VERTICAL
    }

    pub fn is_horizontal(self) -> bool {
        self == NameTableMirroring::HORIZONTAL
    }

    pub fn is_four_screen(self) -> bool {
        // TODO
        false
    }
}

const fn build(quadrants: [VramSide; 4]) -> NameTableMirroring {
    NameTableMirroring { quadrants }
}

impl fmt::Display for NameTableMirroring {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match *self {
            NameTableMirroring::VERTICAL => "Vertical".to_string(),
            NameTableMirroring::HORIZONTAL => "Horizontal".to_string(),
            NameTableMirroring::ONE_SCREEN_LEFT_BANK => "ONE_SCREEN_LEFT_BANK".to_string(),
            NameTableMirroring::ONE_SCREEN_RIGHT_BANK => "ONE_SCREEN_RIGHT_BANK".to_string(),
            _ => todo!("Other NameTableMirrorings including custom."),
        };

        write!(f, "{text}")
    }
}
