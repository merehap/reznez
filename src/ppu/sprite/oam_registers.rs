use crate::ppu::palette::palette_table::PaletteTable;
use crate::ppu::palette::rgbt::Rgbt;
use crate::ppu::sprite::sprite_attributes::{SpriteAttributes, Priority};
use crate::util::bit_util::get_bit;

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
        let rgbt = palette.rgbt_from_low_high(low_bit, high_bit);
        (rgbt, self.attributes.priority(), self.is_sprite_0)
    }
}
