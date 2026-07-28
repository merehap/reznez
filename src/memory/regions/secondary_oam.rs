use ux::u5;

use crate::signals::{Signals, SpriteRenderingDotType};

pub struct SecondaryOam {
    data: [u8; 32],
    index: Option<u5>,
}

impl SecondaryOam {
    pub fn new() -> SecondaryOam {
        SecondaryOam {
            data: [0xFF; 32],
            index: Some(u5::MIN),
        }
    }

    pub fn is_full(&self) -> bool {
        self.index.is_none()
    }

    pub fn current_field(&self) -> SpriteField {
        match to_usize(self.index) % 4 {
            0 => SpriteField::Y,
            1 => SpriteField::Tile,
            2 => SpriteField::Attributes,
            3 => SpriteField::X,
            _ => unreachable!(),
        }
    }

    pub fn oam_register_index(&self) -> usize {
        to_usize(self.index) / 4
    }

    pub fn oam_row_corruption_target(&self, signals: &Signals) -> u5 {
        let mut row = self.index.unwrap_or(u5::MIN);
        if signals.sprite_rendering_dot_type() == SpriteRenderingDotType::Evaluation {
            // "If OAM Corruption occurs between cycles 65 through 256 when the secondary OAM address is not a multiple of 4,
            // it appears to be "ceilinged" to the nearest multiple of 4.
            row = row.wrapping_add(u5::new(3)) & !u5::new(3);
        }

        row
    }

    pub fn peek(&self) -> u8 {
        self.data[to_usize(self.index)]
    }

    pub fn write(&mut self, value: u8) {
        self.data[to_usize(self.index)] = value;
    }

    pub fn reset_index(&mut self) {
        self.index = Some(u5::MIN);
    }

    pub fn advance(&mut self) {
        let Some(index) = &mut self.index else {
            panic!("Can't advance index when Secondary OAM is already full.");
        };

        *index = index.wrapping_add(u5::new(1));
        if *index == u5::MIN {
            // Mark SecondaryOam as full.
            self.index = None;
        }
    }

    // For OAM row corruption purposes
    pub fn peek_head(&self) -> u8 {
        self.data[0]
    }

    // For OAM row corruption purposes
    pub fn write_at(&mut self, index: u5, value: u8) {
        self.data[usize::from(u8::from(index))] = value;
    }
}

fn to_usize(value: Option<u5>) -> usize {
    // If SecondaryOam is full, then it always points at 0.
    u8::from(value.unwrap_or(u5::MIN)).into()
}

#[derive(PartialEq, Eq)]
pub enum SpriteField {
    Y,
    Tile,
    Attributes,
    X,
}