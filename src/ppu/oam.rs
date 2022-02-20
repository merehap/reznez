use crate::memory::memory::PpuMemory;
use crate::ppu::register::registers::ctrl::SpriteHeight;
use crate::ppu::render::frame::Frame;
use crate::ppu::sprite::Sprite;

const ATTRIBUTE_BYTE_INDEX: u8 = 2;

// TODO: OAM should decay:
// https://wiki.nesdev.org/w/index.php?title=PPU_OAM#Dynamic_RAM_decay
pub struct Oam([u8; 256]);

impl Oam {
    pub fn new() -> Oam {
        Oam([0; 256])
    }

    fn sprites(&self) -> [Sprite; 64] {
        let mut iter = self.0.array_chunks::<4>();
        [(); 64].map(|_| {
            let raw = u32::from_be_bytes(*iter.next().unwrap());
            Sprite::from_u32(raw)
        })
    }

    pub fn sprite0(&self) -> Sprite {
        Sprite::from_u32(u32::from_be_bytes(self.0[0..4].try_into().unwrap()))
    }

    pub fn read(&self, index: u8) -> u8 {
        self.0[index as usize]
    }

    pub fn write(&mut self, index: u8, value: u8) {
        // The three unimplemented attribute bits should never be set.
        let value =
            if index % 4 == ATTRIBUTE_BYTE_INDEX {
                value & 0b1110_0011
            } else {
                value
            };
        self.0[index as usize] = value;
    }

    pub fn render_sprites(&self, mem: &PpuMemory, frame: &mut Frame) {
        frame.clear_sprite_buffer();

        let sprite_table_side = mem.regs().sprite_table_side();
        let pattern_table = mem.pattern_table(sprite_table_side);
        let palette_table = mem.palette_table();
        let sprite_height = mem.regs().sprite_height();

        // FIXME: No more sprites will be found once the end of OAM is reached,
        // effectively hiding any sprites before OAM[OAMADDR].
        let sprites = self.sprites();
        // Lower index sprites are drawn on top of higher index sprites.
        for i in (0..sprites.len()).rev() {
            let is_sprite_0 = i == 0;
            if sprite_height == SpriteHeight::Normal {
                sprites[i].render_normal_height(&pattern_table, &palette_table, is_sprite_0, frame);
            } else {
                let sprite = sprites[i];
                let pattern_table =
                    mem.pattern_table(sprite.tall_sprite_pattern_table_side());
                sprite.render_tall(&pattern_table, &palette_table, is_sprite_0, frame);
            }
        }
    }
}
