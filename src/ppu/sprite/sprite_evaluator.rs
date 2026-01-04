use crate::ppu::pixel_index::PixelRow;
use crate::ppu::register::ppu_registers::PpuRegisters;
use crate::ppu::sprite::oam::Oam;

use super::secondary_oam::SecondaryOam;

pub struct SpriteEvaluator {
    oam_data_read: u8,
    secondary_oam: SecondaryOam,
    clear_oam: bool,
    all_sprites_evaluated: bool,
    sprite_0_present: bool,
}

impl SpriteEvaluator {
    pub fn new() -> Self {
        Self {
            oam_data_read: 0,
            secondary_oam: SecondaryOam::new(),
            clear_oam: false,
            all_sprites_evaluated: false,
            sprite_0_present: false,
        }
    }

    pub fn sprite_0_present(&self) -> bool {
        self.sprite_0_present
    }

    pub fn start_clearing_secondary_oam(&mut self) {
        self.secondary_oam.reset_index();
        self.clear_oam = true;
    }

    pub fn start_sprite_evaluation(&mut self) {
        self.secondary_oam.reset_index();
        self.clear_oam = false;
        self.sprite_0_present = false;
    }

    pub fn start_loading_oam_registers(&mut self) {
        self.all_sprites_evaluated = false;
        // TODO: Determine if this needs to occur on cycle 256 instead.
        self.secondary_oam.reset_index();
    }

    pub fn read_oam(&mut self, oam: &Oam, ppu_regs: &PpuRegisters) {
        self.oam_data_read = oam.peek(ppu_regs.oam_addr);
        if self.clear_oam {
            self.oam_data_read = 0xFF;
        }
    }

    pub fn read_secondary_oam_and_advance(&mut self) -> u8 {
        self.secondary_oam.read_and_advance()
    }

    pub fn write_secondary_oam(&mut self, ppu_regs: &mut PpuRegisters) {
        if self.clear_oam {
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
            self.secondary_oam.write(self.oam_data_read);
        }

        if !ppu_regs.oam_addr.new_sprite_started() {
            // The current sprite is in range, copy one more byte of its data over.
            self.secondary_oam.advance();
            self.all_sprites_evaluated = ppu_regs.oam_addr.next_field();
            return;
        }

        // Check if the y coordinate is on screen.
        if let Some(pixel_row) = ppu_regs.clock().scanline_pixel_row()
            && let Some(top_sprite_row) = PixelRow::try_from_u8(self.oam_data_read)
            && let Some(offset) = pixel_row.difference(top_sprite_row)
            && offset < ppu_regs.sprite_height().to_dimension()
        {
            if ppu_regs.oam_addr.is_at_sprite_0() {
                self.sprite_0_present = true;
            }

            if self.secondary_oam.is_full() {
                ppu_regs.set_sprite_overflow();
            }

            self.secondary_oam.advance();
            self.all_sprites_evaluated = ppu_regs.oam_addr.next_field();
            return;
        }

        if self.secondary_oam.is_full() {
            // Sprite overflow hardware bug
            // https://www.nesdev.org/wiki/PPU_sprite_evaluation#Details
            ppu_regs.oam_addr.corrupt_sprite_y_index();
        }

        self.all_sprites_evaluated = ppu_regs.oam_addr.next_sprite();
    }
}
