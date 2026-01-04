use crate::gui::debug_screens::pattern_table::PatternTable;
use crate::gui::debug_screens::sprite::Sprite;
use crate::bus::Bus;
use crate::ppu::pixel_index::PixelRow;
use crate::ppu::sprite::oam::Oam;
use crate::ppu::sprite::sprite_attributes::Priority;
use crate::ppu::render::frame::Frame;
use crate::ppu::sprite::sprite_height::SpriteHeight;

/**
 * DEBUG WINDOW METHODS
 */
impl Oam {
    pub fn only_front_sprites(&self) -> Oam {
        let mut result = self.clone();
        for chunk in result.to_bytes_mut().as_chunks_mut().0.iter_mut() {
            let sprite = Sprite::from_u32(u32::from_be_bytes(*chunk));
            if sprite.priority() == Priority::Behind {
                *chunk = [0xFF, 0, 0, 0];
            }
        }

        result
    }

    pub fn only_back_sprites(&self) -> Oam {
        let mut result = self.clone();
        for chunk in result.to_bytes_mut().as_chunks_mut().0.iter_mut() {
            let sprite = Sprite::from_u32(u32::from_be_bytes(*chunk));
            if sprite.priority() == Priority::InFront {
                *chunk = [0xFF, 0, 0, 0];
            }
        }

        result
    }

    pub fn sprites(&self) -> [Sprite; 64] {
        let mut iter = self.to_bytes().as_chunks().0.iter();
        [(); 64].map(|()| {
            let raw = u32::from_be_bytes(*iter.next().unwrap());
            Sprite::from_u32(raw)
        })
    }

    pub fn render(&self, bus: &Bus, frame: &mut Frame) {
        for pixel_row in PixelRow::iter() {
            self.render_scanline(pixel_row, bus,frame);
        }
    }

    pub fn render_scanline(&self, pixel_row: PixelRow, bus: &Bus, frame: &mut Frame) {
        frame.clear_sprite_line(pixel_row);

        let sprite_table_side = bus.ppu_regs.sprite_table_side();
        let mut pattern_table = PatternTable::from_mem(bus, sprite_table_side);
        let palette_table = bus.palette_table();
        let sprite_height = bus.ppu_regs.sprite_height();

        // FIXME: No more sprites will be found once the end of OAM is reached,
        // effectively hiding any sprites before OAM[OAMADDR].
        let sprites = self.sprites();
        // Lower index sprites are drawn on top of higher index sprites.
        for i in (0..sprites.len()).rev() {
            let is_sprite_0 = i == 0;
            let sprite = sprites[i];
            if sprite_height == SpriteHeight::Tall {
                pattern_table = PatternTable::from_mem(bus, sprite.tile_number().tall_sprite_pattern_table_side());
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