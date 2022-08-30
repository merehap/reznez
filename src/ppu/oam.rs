use crate::memory::memory::PpuMemory;
use crate::ppu::pixel_index::PixelRow;
use crate::ppu::register::registers::ctrl::SpriteHeight;
use crate::ppu::render::frame::Frame;
use crate::ppu::sprite::{Priority, Sprite};

const ATTRIBUTE_BYTE_INDEX: u8 = 2;

// TODO: OAM should decay:
// https://wiki.nesdev.org/w/index.php?title=PPU_OAM#Dynamic_RAM_decay
#[derive(Clone)]
pub struct Oam([u8; 256]);

impl Oam {
    pub fn new() -> Oam {
        Oam([0; 256])
    }

    pub fn only_front_sprites(&self) -> Oam {
        let mut result = self.clone();
        for chunk in result.0.array_chunks_mut::<4>() {
            let sprite = Sprite::from_u32(u32::from_be_bytes(*chunk));
            if sprite.priority() == Priority::Behind {
                *chunk = [0xFF, 0, 0, 0];
            }
        }

        result
    }

    pub fn only_back_sprites(&self) -> Oam {
        let mut result = self.clone();
        for chunk in result.0.array_chunks_mut::<4>() {
            let sprite = Sprite::from_u32(u32::from_be_bytes(*chunk));
            if sprite.priority() == Priority::InFront {
                *chunk = [0xFF, 0, 0, 0];
            }
        }

        result
    }

    pub fn sprites(&self) -> [Sprite; 64] {
        let mut iter = self.0.array_chunks::<4>();
        [(); 64].map(|_| {
            let raw = u32::from_be_bytes(*iter.next().unwrap());
            Sprite::from_u32(raw)
        })
    }

    pub fn read_sprite_data(&self, sprite_index: SpriteIndex, data_index: SpriteDataIndex) -> u8 {
        self.read((sprite_index.to_u8() << 2) | data_index.to_u8())
    }

    pub fn read(&self, index: u8) -> u8 {
        self.0[index as usize]
    }

    pub fn write(&mut self, index: u8, value: u8) {
        // The three unimplemented attribute bits should never be set.
        let value = if index % 4 == ATTRIBUTE_BYTE_INDEX {
            value & 0b1110_0011
        } else {
            value
        };
        self.0[index as usize] = value;
    }

    // For debug windows only.
    pub fn render(&self, mem: &PpuMemory, frame: &mut Frame) {
        for pixel_row in PixelRow::iter() {
            self.render_scanline(pixel_row, mem, frame);
        }
    }

    pub fn render_scanline(
        &self,
        pixel_row: PixelRow,
        mem: &PpuMemory,
        frame: &mut Frame,
    ) {
        frame.clear_sprite_line(pixel_row);

        let sprite_table_side = mem.regs().sprite_table_side();
        let mut pattern_table = mem.pattern_table(sprite_table_side);
        let palette_table = mem.palette_table();
        let sprite_height = mem.regs().sprite_height();

        // FIXME: No more sprites will be found once the end of OAM is reached,
        // effectively hiding any sprites before OAM[OAMADDR].
        let sprites = self.sprites();
        // Lower index sprites are drawn on top of higher index sprites.
        for i in (0..sprites.len()).rev() {
            let is_sprite_0 = i == 0;
            let sprite = sprites[i];
            if sprite_height == SpriteHeight::Tall {
                pattern_table =
                    mem.pattern_table(sprite.tall_sprite_pattern_table_side());
            }

            sprite.render_sliver(
                pixel_row,
                sprite_height,
                &pattern_table,
                &palette_table,
                is_sprite_0,
                frame,
            );
        }
    }
}

pub struct SecondaryOam([u8; 32]);

impl SecondaryOam {
    pub fn new() -> SecondaryOam {
        SecondaryOam([0xFF; 32])
    }

    pub fn blank(&mut self, index: usize) {
        self.0[index] = 0xFF;
    }
}

// "n" in the documentation
#[derive(Clone, Copy)]
pub struct SpriteIndex(u8);

impl SpriteIndex {
    const MAX: u8 = 63;

    pub fn new() -> Self {
        SpriteIndex(0)
    }

    pub fn increment(&mut self) -> bool {
        let overflow = self.0 == SpriteIndex::MAX;
        if overflow {
            self.0 = 0;
        } else {
            self.0 += 1;
        }

        overflow
    }

    fn to_u8(self) -> u8 {
        self.0
    }
}

// "m" in the documentation
#[derive(PartialEq, Clone, Copy)]
pub enum SpriteDataIndex {
    YCoordinate       = 0,
    PatternIndex      = 1,
    Attributes        = 2,
    XCoordinate       = 3,
}

impl SpriteDataIndex {
    pub fn new() -> Self {
        SpriteDataIndex::YCoordinate
    }

    pub fn increment(&mut self) -> bool {
        use SpriteDataIndex::*;
        *self = match self {
            YCoordinate       => PatternIndex,
            PatternIndex      => Attributes,
            Attributes        => XCoordinate,
            XCoordinate       => YCoordinate,
        };

        // Did we overflow?
        *self == YCoordinate
    }

    fn to_u8(self) -> u8 {
        self as u8
    }
}
