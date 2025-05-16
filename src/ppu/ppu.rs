use log::{info, log_enabled};
use log::Level::Info;

use crate::memory::memory::{PpuMemory, SignalLevel};
use crate::memory::ppu::ppu_address::PpuAddress;
use crate::ppu::cycle_action::cycle_action::CycleAction;
use crate::ppu::cycle_action::frame_actions::{FrameActions, NTSC_FRAME_ACTIONS};
use crate::ppu::palette::palette_table_index::PaletteTableIndex;
use crate::ppu::palette::rgbt::Rgbt;
use crate::ppu::pattern_table::PatternIndex;
use crate::ppu::pattern_table::PatternTableSide;
use crate::ppu::pixel_index::{PixelIndex, PixelRow};
use crate::ppu::register::registers::attribute_register::AttributeRegister;
use crate::ppu::register::registers::pattern_register::PatternRegister;
use crate::ppu::render::frame::Frame;
use crate::ppu::sprite::sprite_attributes::SpriteAttributes;
use crate::ppu::sprite::secondary_oam::SecondaryOam;
use crate::ppu::sprite::oam_registers::OamRegisters;
use crate::ppu::sprite::sprite_y::SpriteY;
use crate::ppu::sprite::sprite_height::SpriteHeight;

use super::palette::bank_color_assigner::BankColorAssigner;

pub struct Ppu {
    oam_data_read: u8,
    secondary_oam: SecondaryOam,
    oam_registers: OamRegisters,
    oam_register_index: usize,
    clear_oam: bool,
    all_sprites_evaluated: bool,

    next_pattern_index: PatternIndex,
    pattern_register: PatternRegister,
    attribute_register: AttributeRegister,

    next_sprite_pattern_index: PatternIndex,
    current_sprite_y: SpriteY,
    sprite_0_present: bool,
    // TODO: Remove this. The IO bus should be set to this instead.
    pattern_address: PpuAddress,
    sprite_visible: bool,

    frame_actions: FrameActions,

    pattern_source_frame: Frame,
    bank_color_assigner: BankColorAssigner,
}

impl Ppu {
    pub fn new(memory: &PpuMemory) -> Ppu {
        Ppu {
            oam_data_read: 0,
            secondary_oam: SecondaryOam::new(),
            oam_registers: OamRegisters::new(),
            oam_register_index: 0,
            clear_oam: false,
            all_sprites_evaluated: false,

            next_pattern_index: PatternIndex::new(0),
            pattern_register: PatternRegister::new(),
            attribute_register: AttributeRegister::new(),

            next_sprite_pattern_index: PatternIndex::new(0),
            current_sprite_y: SpriteY::new(0),
            sprite_0_present: false,
            pattern_address: PpuAddress::ZERO,
            sprite_visible: false,

            frame_actions: NTSC_FRAME_ACTIONS.clone(),

            pattern_source_frame: Frame::new(),
            bank_color_assigner: BankColorAssigner::new(memory),
        }
    }

    pub fn step(&mut self, mem: &mut PpuMemory, frame: &mut Frame) {
        let clock = *mem.regs().clock();
        mem.regs_mut().maybe_toggle_rendering_enabled();
        mem.regs_mut().maybe_decay_ppu_io_bus(&clock);

        if log_enabled!(target: "ppusteps", Info) {
            info!(" {clock}\t{}", self.frame_actions.format_current_cycle_actions(&clock));
        }

        // TODO: Figure out how to eliminate duplication and the index.
        let len = self.frame_actions.current_cycle_actions(&clock).len();
        for i in 0..len {
            let cycle_action = self.frame_actions.current_cycle_actions(&clock)[i];
            self.execute_cycle_action(mem, frame, cycle_action);
        }

        if mem.regs().can_generate_nmi() {
            mem.set_nmi_line_level(SignalLevel::Low);
        } else {
            mem.set_nmi_line_level(SignalLevel::High);
        }

        mem.process_end_of_ppu_cycle();
    }

    pub fn execute_cycle_action(
        &mut self,
        mem: &mut PpuMemory,
        frame: &mut Frame,
        cycle_action: CycleAction,
    ) {
        let background_table_side = mem.regs().background_table_side();

        let tile_column = mem.regs().current_address.x_scroll().coarse();
        let tile_row = mem.regs().current_address.y_scroll().coarse();
        let row_in_tile = mem.regs().current_address.y_scroll().fine();
        let name_table_quadrant = mem.regs().current_address.name_table_quadrant();

        let background_enabled = mem.regs().background_enabled();
        let sprites_enabled = mem.regs().sprites_enabled();

        let clock = *mem.regs().clock();

        use CycleAction::*;
        match cycle_action {
            GetPatternIndex => {
                if !mem.regs().rendering_enabled() { return; }
                let address = PpuAddress::in_name_table(name_table_quadrant, tile_column, tile_row);
                self.next_pattern_index = PatternIndex::new(mem.read(address).value());
            }
            GetPaletteIndex => {
                if !mem.regs().rendering_enabled() { return; }
                let address = PpuAddress::in_attribute_table(name_table_quadrant, tile_column, tile_row);
                let attribute_byte = mem.read(address).value();
                let palette_table_index =
                    PaletteTableIndex::from_attribute_byte(attribute_byte, tile_column, tile_row);
                self.attribute_register.set_pending_palette_table_index(palette_table_index);
            }
            LoadPatternLowAddress => {
                self.pattern_address = PpuAddress::in_pattern_table(
                    background_table_side, self.next_pattern_index, row_in_tile, false);
                mem.trigger_ppu_address_change(self.pattern_address);
            }
            LoadPatternHighAddress => {
                self.pattern_address = PpuAddress::in_pattern_table(
                    background_table_side, self.next_pattern_index, row_in_tile, true);
                mem.trigger_ppu_address_change(self.pattern_address);
            }
            GetPatternLowByte => {
                if !mem.regs().rendering_enabled() { return; }
                self.pattern_register.set_pending_low_byte(mem.read(self.pattern_address));
            }
            GetPatternHighByte => {
                if !mem.regs().rendering_enabled() { return; }
                self.pattern_register.set_pending_high_byte(mem.read(self.pattern_address));
            }

            GotoNextTileColumn => {
                if !mem.regs().rendering_enabled() { return; }
                mem.regs_mut().current_address.increment_coarse_x_scroll();
            }
            GotoNextPixelRow => {
                if !mem.regs().rendering_enabled() { return; }
                mem.regs_mut().current_address.increment_fine_y_scroll();
            }
            ResetTileColumn => {
                if !mem.regs().rendering_enabled() { return; }
                let next_address = mem.regs().next_address;
                mem.regs_mut().current_address.copy_x_scroll(next_address);
                mem.regs_mut().current_address.copy_horizontal_name_table_side(next_address);
            }
            PrepareForNextTile => {
                if !mem.regs().rendering_enabled() { return; }
                self.attribute_register.prepare_next_palette_table_index();
                self.pattern_register.load_next_palette_indexes();
            }
            SetPixel => {
                let (pixel_column, pixel_row) = PixelIndex::try_from_clock(&clock).unwrap().to_column_row();
                // TODO: Verify if these need to be delayed like self.rendering_enabled.
                if background_enabled {
                    // TODO: Figure out where this goes. Maybe have frame call palette_table when displaying.
                    frame.set_universal_background_rgb(
                        mem.palette_table().universal_background_rgb(),
                    );

                    let column_in_tile = mem.regs_mut().current_address.x_scroll().fine();
                    let palette_table_index = self.attribute_register.palette_table_index(column_in_tile);
                    let palette = mem.palette_table().background_palette(palette_table_index);

                    let background_pixel = self.pattern_register
                        .palette_index(column_in_tile)
                        .map_or(Rgbt::Transparent, |palette_index| Rgbt::Opaque(palette[palette_index]));

                    frame.set_background_pixel(pixel_column, pixel_row, background_pixel);

                    let bank_pixel = if background_pixel.is_transparent() {
                        Rgbt::Transparent
                    } else {
                        let rgb = self.bank_color_assigner.rgb_for_source(self.pattern_register.current_peek().source());
                        Rgbt::Opaque(rgb)
                    };
                    self.pattern_source_frame.set_background_pixel(pixel_column, pixel_row, bank_pixel);
                }

                // TODO: Verify if this need to be delayed like self.rendering_enabled.
                if sprites_enabled {
                    let (sprite_pixel, priority, is_sprite_0, ppu_peek) = self.oam_registers.step(&mem.palette_table());
                    frame.set_sprite_pixel(
                        pixel_column,
                        pixel_row,
                        sprite_pixel,
                        priority,
                        is_sprite_0,
                    );

                    let bank_pixel = if sprite_pixel.is_transparent() {
                        Rgbt::Transparent
                    } else {
                        let rgb = self.bank_color_assigner.rgb_for_source(ppu_peek.source());
                        Rgbt::Opaque(rgb)
                    };
                    self.pattern_source_frame.set_sprite_pixel(
                        pixel_column,
                        pixel_row,
                        bank_pixel,
                        priority,
                        false,
                    );
                }

                // https://wiki.nesdev.org/w/index.php?title=PPU_OAM#Sprite_zero_hits
                // TODO: Verify if these need to be delayed like self.rendering_enabled.
                if sprites_enabled && background_enabled
                    && frame.pixel(mem.regs().mask(), pixel_column, pixel_row).1.hit()
                {
                    mem.regs_mut().set_sprite0_hit();
                }
            }
            PrepareForNextPixel => {
                // TODO: Verify if this needs to be !self.rendering_enabled, which is time-delayed.
                if !background_enabled { return; }
                self.pattern_register.shift_left();
                self.attribute_register.push_next_palette_table_index();
            }

            MaybeCorruptOamStart => {
                // Unclear if these are the correct cycles to trigger on.
                if mem.regs().rendering_enabled() {
                    let oam_addr = mem.regs().oam_addr;
                    let cycle = mem.regs().clock().cycle();
                    mem.oam_mut().maybe_corrupt_starting_byte(oam_addr, cycle);
                }
            }

            ResetOamAddress => {
                if !mem.regs().rendering_enabled() { return; }
                mem.regs_mut().oam_addr.reset();
            }

            ReadOamByte => {
                if !mem.regs().rendering_enabled() { return; }
                // This is a dummy read if OAM clear is active. TODO: Can this be removed?
                self.oam_data_read = mem.oam().peek(mem.regs().oam_addr);
                if self.clear_oam {
                    self.oam_data_read = 0xFF;
                }
            }
            WriteSecondaryOamByte => {
                if !mem.regs().rendering_enabled() { return; }

                if self.clear_oam {
                    self.secondary_oam.write(self.oam_data_read);
                    self.secondary_oam.advance();
                    return;
                }

                if self.all_sprites_evaluated {
                    // TODO: Reading and incrementing still happen after sprite evaluation is
                    // complete, but writes fail (i.e. they don't happen).
                    self.oam_data_read = self.secondary_oam.read();
                    return;
                }

                if !self.secondary_oam.is_full() {
                    self.secondary_oam.write(self.oam_data_read);
                }

                if !mem.regs().oam_addr.new_sprite_started() {
                    // The current sprite is in range, copy one more byte of its data over.
                    self.secondary_oam.advance();
                    self.all_sprites_evaluated = mem.regs_mut().oam_addr.next_field();
                    return;
                }

                // Check if the y coordinate is on screen.
                if let Some(pixel_row) = mem.regs().clock().scanline_pixel_row()
                    && let Some(top_sprite_row) = PixelRow::try_from_u8(self.oam_data_read)
                    && let Some(offset) = pixel_row.difference(top_sprite_row)
                    && offset < (mem.regs().sprite_height().to_dimension())
                {
                    if mem.regs().oam_addr.is_at_sprite_0() {
                        self.sprite_0_present = true;
                    }

                    if self.secondary_oam.is_full() {
                        mem.regs_mut().set_sprite_overflow();
                    }

                    self.secondary_oam.advance();
                    self.all_sprites_evaluated = mem.regs_mut().oam_addr.next_field();
                    return;
                }

                if self.secondary_oam.is_full() {
                    // Sprite overflow hardware bug
                    // https://www.nesdev.org/wiki/PPU_sprite_evaluation#Details
                    mem.regs_mut().oam_addr.corrupt_sprite_y_index();
                }

                self.all_sprites_evaluated = mem.regs_mut().oam_addr.next_sprite();
            }
            ReadSpriteY => {
                if !mem.regs().rendering_enabled() { return; }
                self.current_sprite_y = SpriteY::new(self.secondary_oam.read_and_advance());
            }
            ReadSpritePatternIndex => {
                if !mem.regs().rendering_enabled() { return; }
                self.next_sprite_pattern_index = PatternIndex::new(self.secondary_oam.read_and_advance());
            }
            ReadSpriteAttributes => {
                if !mem.regs().rendering_enabled() { return; }

                let attributes = SpriteAttributes::from_u8(self.secondary_oam.read_and_advance());
                self.oam_registers.registers[self.oam_register_index].set_attributes(attributes);
            }
            ReadSpriteX => {
                if !mem.regs().rendering_enabled() { return; }

                let x_counter = self.secondary_oam.read_and_advance();
                self.oam_registers.registers[self.oam_register_index].set_x_counter(x_counter);
            }
            DummyReadSpriteX => {
                // TODO
                //if !self.rendering_enabled { return; }
            }

            LoadSpritePatternLowAddress => {
                let select_high = false;
                (self.pattern_address, self.sprite_visible) =
                    self.current_sprite_pattern_address(mem, select_high);
                mem.trigger_ppu_address_change(self.pattern_address);
            }
            LoadSpritePatternHighAddress => {
                let select_high = true;
                (self.pattern_address, self.sprite_visible) =
                    self.current_sprite_pattern_address(mem, select_high);
                mem.trigger_ppu_address_change(self.pattern_address);
            }
            GetSpritePatternLowByte => {
                if mem.regs().rendering_enabled() {
                    let pattern_low = mem.read(self.pattern_address);
                    if self.sprite_visible {
                        self.oam_registers.registers[self.oam_register_index]
                            .set_pattern_low(pattern_low);
                    }
                }
            }
            GetSpritePatternHighByte => {
                if mem.regs().rendering_enabled() {
                    let pattern_high = mem.read(self.pattern_address);
                    if self.sprite_visible {
                        self.oam_registers.registers[self.oam_register_index]
                            .set_pattern_high(pattern_high);
                    }
                }
            }
            IncrementOamRegisterIndex => {
                self.oam_register_index += 1;
            }

            StartVisibleScanlines => {
                info!(target: "ppustage", "{}\tVISIBLE SCANLINES", mem.regs().clock());
            }
            StartPostRenderScanline => {
                info!(target: "ppustage", "{}\tPOST-RENDER SCANLINE", mem.regs().clock());
            }
            StartVblankScanlines => {
                info!(target: "ppustage", "{}\tVBLANK SCANLINES", mem.regs().clock());
            }
            StartPreRenderScanline => {
                info!(target: "ppustage", "{}\tPRE-RENDER SCANLINE", mem.regs().clock());
            }

            StartReadingBackgroundTiles => {
                info!(target: "ppustage", "{}\t\tREADING BACKGROUND TILES", mem.regs().clock());
            }
            StopReadingBackgroundTiles => {
                info!(target: "ppustage", "{}\t\tENDED READING BACKGROUND TILES", mem.regs().clock());
            }

            StartClearingSecondaryOam => {
                info!(target: "ppustage", "{}\t\tCLEARING SECONDARY OAM", mem.regs().clock());
                self.secondary_oam.reset_index();
                self.clear_oam = true;
            }
            StartSpriteEvaluation => {
                info!(target: "ppustage", "\t\tSPRITE EVALUATION");
                self.secondary_oam.reset_index();
                self.clear_oam = false;
                self.oam_register_index = 0;
                self.sprite_0_present = false;
            }
            StartLoadingOamRegisters => {
                info!(target: "ppustage", "\t\tLoading OAM registers.");
                self.all_sprites_evaluated = false;
                // TODO: Determine if this needs to occur on cycle 256 instead.
                self.secondary_oam.reset_index();
                self.oam_registers.set_sprite_0_presence(self.sprite_0_present);
            }
            StopLoadingOamRegisters => {
                info!(target: "ppustage", "\t\tLoading OAM registers ended.");
            }

            StartVblank => {
                if mem.regs().suppress_vblank_active {
                    info!(target: "ppuflags", " {}\tSuppressing vblank.", mem.regs().clock());
                } else {
                    let clock = *mem.regs().clock();
                    mem.regs_mut().start_vblank(&clock);
                }

                mem.regs_mut().suppress_vblank_active = false;
            }
            SetInitialScrollOffsets => {
                // TODO: Verify if this needs to be !self.rendering_enabled, which is time-delayed.
                if !background_enabled { return; }
                mem.regs_mut().current_address = mem.regs().next_address;
            }
            SetInitialYScroll => {
                // TODO: Verify if this needs to be !self.rendering_enabled, which is time-delayed.
                if !background_enabled { return; }
                let next_address = mem.regs().next_address;
                mem.regs_mut().current_address.copy_y_scroll(next_address);
            }

            ClearFlags => {
                let clock = *mem.regs().clock();
                mem.regs_mut().stop_vblank(&clock);
                mem.regs_mut().clear_sprite0_hit();
                mem.regs_mut().clear_sprite_overflow();
            }
        }
    }

    pub fn pattern_source_frame(&self) -> &Frame {
        &self.pattern_source_frame
    }

    fn current_sprite_pattern_address(&self, mem: &PpuMemory, select_high: bool) -> (PpuAddress, bool) {
        let sprite_table_side = mem.regs().sprite_table_side();
        let sprite_height = mem.regs().sprite_height();
        let sprite_table_side = match sprite_height {
            SpriteHeight::Normal => sprite_table_side,
            SpriteHeight::Tall => self.next_sprite_pattern_index.tall_sprite_pattern_table_side(),
        };

        let address;
        let visible;
        if let Some(pixel_row) = mem.regs().clock().scanline_pixel_row() {
            let attributes = self.oam_registers.registers[self.oam_register_index].attributes();
            if let Some((pattern_index, row_in_half, v)) = self.next_sprite_pattern_index.index_and_row(
                self.current_sprite_y,
                attributes.flip_vertically(),
                sprite_height,
                pixel_row
            ) {
                visible = v;
                address = PpuAddress::in_pattern_table(
                    sprite_table_side, pattern_index, row_in_half, select_high);
            } else {
                // Sprite not on current scanline. TODO: what address should be here?
                if sprite_table_side == PatternTableSide::Left {
                    address = PpuAddress::from_u16(0x0000);
                } else {
                    address = PpuAddress::from_u16(0x1000);
                }
                visible = false;
            }
        } else {
            // Pre-render scanline. TODO: use correct address based upon pattern index.
            if sprite_table_side == PatternTableSide::Left {
                address = PpuAddress::from_u16(0x0000);
            } else {
                address = PpuAddress::from_u16(0x1000);
            }
            visible = false;
        }

        (address, visible)
    }
}
