use crate::memory::memory::PpuMemory;
use crate::ppu::palette::rgbt::Rgbt;
use crate::ppu::palette::palette_index::PaletteIndex;
use crate::ppu::palette::palette_table::PaletteTable;
use crate::ppu::pixel_index::PixelRow;
use crate::ppu::register::registers::ctrl::SpriteHeight;
use crate::ppu::render::frame::Frame;
use crate::ppu::sprite::{Priority, Sprite, SpriteAttributes};
use crate::util::bit_util::get_bit;

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

    pub fn read_sprite_data(&self, oam_index: OamIndex) -> u8 {
        self.read((oam_index.sprite_index << 2) | oam_index.field_index as u8)
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
                    mem.pattern_table(sprite.pattern_index().tall_sprite_pattern_table_side());
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

pub struct SecondaryOam {
    data: [u8; 32],
    index: usize,
    is_full: bool,
}

impl SecondaryOam {
    pub fn new() -> SecondaryOam {
        SecondaryOam {
            data: [0xFF; 32],
            index: 0,
            is_full: false,
        }
    }

    pub fn is_full(&self) -> bool {
        self.is_full
    }

    pub fn read_and_advance(&mut self) -> u8 {
        let result = self.data[self.index];
        self.advance();
        result
    }

    pub fn write(&mut self, value: u8) {
        self.data[self.index] = value;
    }

    pub fn write_and_advance(&mut self, value: u8) {
        if !self.is_full {
            self.data[self.index] = value;
        }

        self.advance();
    }

    pub fn reset_index(&mut self) {
        self.index = 0;
        self.is_full = false;
    }

    pub fn advance(&mut self) {
        if self.index == 31 {
            self.index = 0;
            self.is_full = true;
        } else {
            self.index += 1;
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct OamIndex {
    // "n" in the documentation
    sprite_index: u8,
    // "m" in the documentation
    field_index: FieldIndex,
    // Buggy sprite overflow offset.
    sprite_start_field_index: FieldIndex,
    end_reached: bool,
}

impl OamIndex {
    const MAX_SPRITE_INDEX: u8 = 63;

    pub fn new() -> OamIndex {
        OamIndex {
            sprite_index: 0,
            field_index: FieldIndex::YCoordinate,
            // This field keeps its initial value unless a sprite overflow occurs.
            sprite_start_field_index: FieldIndex::YCoordinate,
            end_reached: false,
        }
    }

    pub fn new_sprite_started(self) -> bool {
        self.field_index == self.sprite_start_field_index
    }

    pub fn end_reached(self) -> bool {
        self.end_reached
    }

    pub fn is_at_sprite_0(self) -> bool {
        self.sprite_index == 0
    }

    pub fn reset(&mut self) {
        *self = OamIndex::new();
    }

    pub fn next_sprite(&mut self) {
        if self.sprite_index == OamIndex::MAX_SPRITE_INDEX {
            self.end_reached = true;
        }

        if self.end_reached {
            self.sprite_index = 0;
        } else {
            self.sprite_index += 1;
        }
    }

    pub fn next_field(&mut self) {
        self.field_index.increment();
        let carry = self.field_index == FieldIndex::YCoordinate;
        if carry {
            self.next_sprite();
        }
    }

    pub fn corrupt_sprite_y_index(&mut self) {
        self.field_index.increment();
        self.sprite_start_field_index = self.field_index;
    }
}

#[derive(PartialEq, Clone, Copy, Debug)]
enum FieldIndex {
    YCoordinate  = 0,
    PatternIndex = 1,
    Attributes   = 2,
    XCoordinate  = 3,
}

impl FieldIndex {
    pub fn increment(&mut self) {
        use FieldIndex::*;
        *self = match self {
            YCoordinate  => PatternIndex,
            PatternIndex => Attributes,
            Attributes   => XCoordinate,
            XCoordinate  => YCoordinate,
        };
    }
}

pub struct OamRegisters {
    pub registers: [SpriteRegisters; 8],
}

impl OamRegisters {
    pub fn new() -> OamRegisters {
        OamRegisters {
            registers: [SpriteRegisters::new(); 8],
        }
    }

    pub fn set_sprite_0_presence(&mut self, present: bool) {
        self.registers[0].is_sprite_0 = present;
    }

    pub fn step(&mut self, palette_table: &PaletteTable) -> (Rgbt, Priority, bool) {
        let mut result = (Rgbt::Transparent, Priority::Behind, false);
        for register in self.registers.iter_mut().rev() {
            let candidate@(rgbt, _, _) = register.step(palette_table);
            if let Rgbt::Opaque(_) = rgbt {
                result = candidate;
            }
        }

        result
    }
}


#[derive(Clone, Copy)]
pub struct SpriteRegisters {
    low_pattern: u8,
    high_pattern: u8,
    attributes: SpriteAttributes,
    x_counter: u8,
    is_sprite_0: bool,
}

impl SpriteRegisters {
    pub fn new() -> SpriteRegisters {
        SpriteRegisters {
            low_pattern: 0,
            high_pattern: 0,
            attributes: SpriteAttributes::new(),
            x_counter: 0,
            is_sprite_0: false,
        }
    }

    // TODO: Store PatternIndex and set patterns later on.
    pub fn set_pattern(&mut self, low_pattern: u8, high_pattern: u8) {
        self.low_pattern = low_pattern;
        self.high_pattern = high_pattern;
    }

    pub fn set_attributes(&mut self, attributes: SpriteAttributes) {
        self.attributes = attributes;
    }

    pub fn set_x_counter(&mut self, initial_value: u8) {
        self.x_counter = initial_value;
    }

    pub fn step(&mut self, palette_table: &PaletteTable) -> (Rgbt, Priority, bool) {
        if self.x_counter > 0 {
            // This sprite is still inactive.
            self.x_counter -= 1;

            return (Rgbt::Transparent, Priority::Behind, false);
        }

        // Ugly :-(
        let low_bit;
        let high_bit;
        if self.attributes.flip_horizontally() {
            low_bit = get_bit(self.low_pattern, 7);
            high_bit = get_bit(self.high_pattern, 7);
            self.low_pattern >>= 1;
            self.high_pattern >>= 1;
        } else {
            low_bit = get_bit(self.low_pattern, 0);
            high_bit = get_bit(self.high_pattern, 0);
            self.low_pattern <<= 1;
            self.high_pattern <<= 1;
        }

        let palette = palette_table.sprite_palette(self.attributes.palette_table_index());
        let rgbt = match (low_bit, high_bit) {
            (false, false) => Rgbt::Transparent,
            (true, false) => Rgbt::Opaque(palette[PaletteIndex::One]),
            (false, true) => Rgbt::Opaque(palette[PaletteIndex::Two]),
            (true, true) => Rgbt::Opaque(palette[PaletteIndex::Three]),
        };

        (rgbt, self.attributes.priority(), self.is_sprite_0)
    }
}
