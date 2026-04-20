use crate::ppu::pixel_index::PixelRow;
use crate::ppu::ppu_clock::PpuClock;
use crate::ppu::register::ppu_registers::PpuRegisters;
use crate::ppu::sprite::oam::Oam;
use crate::ppu::sprite::secondary_oam::SpriteField;

use super::secondary_oam::SecondaryOam;

pub struct SpriteEvaluator {
    oam_data_read: u8,
    secondary_oam: SecondaryOam,
    all_sprites_evaluated: bool,
    sprite_0_present: bool,
}

impl SpriteEvaluator {
    pub fn new() -> Self {
        Self {
            oam_data_read: 0,
            secondary_oam: SecondaryOam::new(),
            all_sprites_evaluated: false,
            sprite_0_present: false,
        }
    }

    pub fn sprite_0_present(&self) -> bool {
        self.sprite_0_present
    }

    pub fn start_clearing_secondary_oam(&mut self) {
        self.secondary_oam.reset_index();
    }

    pub fn start_sprite_evaluation(&mut self) {
        self.secondary_oam.reset_index();
        self.sprite_0_present = false;
        self.all_sprites_evaluated = false;
    }

    pub fn start_loading_oam_registers(&mut self) {
        // TODO: Determine if this needs to occur on cycle 256 instead.
        self.secondary_oam.reset_index();
    }

    pub fn read_oam(&mut self, oam: &mut Oam, clock: &PpuClock, ppu_regs: &PpuRegisters) {
        self.oam_data_read = oam.read(clock, ppu_regs.oam_addr, ppu_regs.rendering_enabled());
    }

    pub fn read_secondary_oam_and_advance(&mut self) -> u8 {
        self.secondary_oam.read_and_advance()
    }

    pub fn write_secondary_oam(&mut self, clock: &PpuClock, ppu_regs: &mut PpuRegisters) {
        if clock.is_oam_clearing_window() {
            self.secondary_oam.write(self.oam_data_read);
            self.secondary_oam.advance();
            return;
        }

        if self.all_sprites_evaluated {
            // TODO: Reading and incrementing still happen after sprite evaluation is
            // complete, but writes fail (i.e. they don't happen).
            self.oam_data_read = self.secondary_oam.peek();
            return;
        }

        if !self.secondary_oam.is_full() {
            // Copy the sprite byte into secondary OAM, but it may be overwritten
            // if it is a Y coordinate that is not in range.
            self.secondary_oam.write(self.oam_data_read);
        }

        // If the Y-index is not currently being evaluated, then the current sprite was already verified to be on screen,
        // so copy another byte of its data over.
        if self.secondary_oam.current_field() != SpriteField::Y {
            self.secondary_oam.advance();
            self.all_sprites_evaluated = ppu_regs.oam_addr.next_field();
            return;
        }

        // Check if the sprite's Y-index is on screen and if the current row is inside of the sprite.
        if let Some(pixel_row) = clock.scanline_pixel_row()
            && let Some(top_sprite_row) = PixelRow::try_from_u8(self.oam_data_read)
            && let Some(offset) = pixel_row.difference(top_sprite_row)
            && offset < ppu_regs.sprite_height().to_dimension()
        {
            if clock.cycle() == 66 {
                self.sprite_0_present = true;
            }

            if self.secondary_oam.is_full() {
                ppu_regs.sprite_overflow = true;
            }

            self.secondary_oam.advance();
            self.all_sprites_evaluated = ppu_regs.oam_addr.next_field();
            return;
        }

        // If the sprite isn't in range and OAM is full, then corrupt the OAMADDR.
        // Sprite overflow hardware bug: https://www.nesdev.org/wiki/PPU_sprite_evaluation#Details
        if self.secondary_oam.is_full() {
            ppu_regs.oam_addr.corrupt_sprite_y_index();
        }

        // If the sprite isn't in range, move to the next sprite.
        self.all_sprites_evaluated = ppu_regs.oam_addr.next_sprite();
    }
}