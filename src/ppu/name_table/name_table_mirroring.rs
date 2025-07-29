use std::fmt;

use crate::memory::ppu::ciram::CiramSide;
use crate::ppu::name_table::name_table_quadrant::NameTableQuadrant;

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct NameTableMirroring {
    // TopLeft, TopRight, BottomLeft, BottomRight
    quadrants: [NameTableSource; 4],
}

impl NameTableMirroring {
    pub const VERTICAL: NameTableMirroring =
        build([CiramSide::Left, CiramSide::Right, CiramSide::Left, CiramSide::Right]);
    pub const HORIZONTAL: NameTableMirroring =
        build([CiramSide::Left, CiramSide::Left, CiramSide::Right, CiramSide::Right]);
    pub const ONE_SCREEN_LEFT_BANK: NameTableMirroring =
        build([CiramSide::Left, CiramSide::Left, CiramSide::Left, CiramSide::Left]);
    pub const ONE_SCREEN_RIGHT_BANK: NameTableMirroring =
        build([CiramSide::Right, CiramSide::Right, CiramSide::Right, CiramSide::Right]);

    pub const fn new(
        top_left_quadrant: NameTableSource,
        top_right_quadrant: NameTableSource,
        bottom_left_quadrant: NameTableSource,
        bottom_right_quadrant: NameTableSource,
    ) -> Self {
        Self { quadrants: [top_left_quadrant, top_right_quadrant, bottom_left_quadrant, bottom_right_quadrant] }
    }

    pub fn name_table_source_in_quadrant(self, quadrant: NameTableQuadrant) -> NameTableSource {
        self.quadrants[quadrant as usize]
    }

    pub fn quadrants(self) -> [NameTableSource; 4] {
        self.quadrants
    }

    pub fn set_quadrant(&mut self, quadrant: NameTableQuadrant, side: CiramSide) {
        self.set_quadrant_to_source(quadrant, NameTableSource::Ciram(side));
    }

    pub fn set_quadrant_to_source(&mut self, quadrant: NameTableQuadrant, source: NameTableSource) {
        self.quadrants[quadrant as usize] = source;
    }

    pub fn is_vertical(self) -> bool {
        self == Self::VERTICAL
    }

    pub fn is_horizontal(self) -> bool {
        self == Self::HORIZONTAL
    }

    pub fn is_regular_one_screen(self) -> bool {
        self == Self::ONE_SCREEN_LEFT_BANK || self == Self::ONE_SCREEN_RIGHT_BANK
    }

    pub fn is_four_screen(self) -> bool {
        // TODO
        false
    }
}

const fn build(quadrants: [CiramSide; 4]) -> NameTableMirroring {
    NameTableMirroring {
        quadrants: [
            NameTableSource::Ciram(quadrants[0]),
            NameTableSource::Ciram(quadrants[1]),
            NameTableSource::Ciram(quadrants[2]),
            NameTableSource::Ciram(quadrants[3]),
        ]
    }
}

impl fmt::Display for NameTableMirroring {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match *self {
            NameTableMirroring::VERTICAL => "Vertical".to_string(),
            NameTableMirroring::HORIZONTAL => "Horizontal".to_string(),
            NameTableMirroring::ONE_SCREEN_LEFT_BANK => "OneScreenLeftBank".to_string(),
            NameTableMirroring::ONE_SCREEN_RIGHT_BANK => "OneScreenRightBank".to_string(),
            NameTableMirroring { quadrants: [top_left, top_right, bottom_left, bottom_right] } =>
                format!("[{top_left}, {top_right}, {bottom_left}, {bottom_right}]"),
        };

        write!(f, "{text}")
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum NameTableSource {
    Ciram(CiramSide),
    SaveRam(u32),
    ExtendedRam,
    FillModeTile,
}

impl fmt::Display for NameTableSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            NameTableSource::Ciram(CiramSide::Left) => "CIRAM(A)",
            NameTableSource::Ciram(CiramSide::Right) => "CIRAM(B)",
            NameTableSource::SaveRam(index) => &format!("SRAM@{index:04X}"),
            NameTableSource::ExtendedRam => "ExtRAM",
            NameTableSource::FillModeTile => "FillMode",
        };

        write!(f, "{text}")
    }
}
