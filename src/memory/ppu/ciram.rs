// Clippy bug.
#![allow(clippy::needless_borrow)]

use crate::memory::bank::bank_number::WriteStatus;
use crate::util::unit::KIBIBYTE;

const CIRAM_SIZE: usize = 2 * KIBIBYTE as usize;
const SIDE_SIZE: usize = KIBIBYTE as usize;

// Console-internal name table RAM.
// TODO: Is CIRAM disabled again upon soft reset?
pub struct Ciram {
    raw: Box<[u8; CIRAM_SIZE]>,
    write_status: WriteStatus,
}

impl Ciram {
    pub fn new() -> Self {
        Self {
            raw: Box::new([0; CIRAM_SIZE]),
            // Can't write to CIRAM until the PPU has initialized a bit.
            write_status: WriteStatus::Disabled,
        }
    }

    pub fn side(&self, side: CiramSide) -> &[u8; SIDE_SIZE] {
        let start_index = side as usize;
        self.raw[start_index..start_index + SIDE_SIZE]
            .try_into()
            .unwrap()
    }

    pub fn enable_writes(&mut self) {
        self.write_status = WriteStatus::Enabled;
    }

    pub fn disable_writes(&mut self) {
        self.write_status = WriteStatus::Disabled;
    }

    pub fn write(&mut self, side: CiramSide, index: u16, value: u8) {
        if self.write_status == WriteStatus::Enabled {
            self.raw[usize::from(side as u16 + index)] = value;
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug, Default)]
pub enum CiramSide {
    #[default]
    Left = 0,
    Right = SIDE_SIZE as isize,
}

impl CiramSide {
    pub fn from_page_number(page_number: u16) -> Self {
        match page_number {
            0 => Self::Left,
            1 => Self::Right,
            _ => panic!("Bad page_number for CIRAM side. Must be 0 or 1, but was {page_number}.")
        }
    }

    pub fn to_page_number(self) -> u16 {
        match self {
            Self::Left => 0,
            Self::Right => 1,
        }
    }
}