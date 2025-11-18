use log::{info, log_enabled};
use log::Level::Info;

use crate::mapper::Mapper;
use crate::memory::memory::Memory;
use crate::memory::ppu::ppu_address::PpuAddress;
use crate::memory::signal_level::SignalLevel;
use crate::ppu::cycle_action::cycle_action::CycleAction;
use crate::ppu::cycle_action::frame_actions::{FrameActions, NTSC_FRAME_ACTIONS};
use crate::ppu::palette::rgbt::Rgbt;
use crate::ppu::pattern_table_side::PatternTableSide;
use crate::ppu::pixel_index::{PixelIndex, PixelRow};
use crate::ppu::register::ppu_registers::Toggle;
use crate::ppu::register::registers::attribute_register::AttributeRegister;
use crate::ppu::register::registers::pattern_register::PatternRegister;
use crate::ppu::render::frame::Frame;
use crate::ppu::sprite::sprite_attributes::SpriteAttributes;
use crate::ppu::sprite::oam_registers::OamRegisters;
use crate::ppu::sprite::sprite_y::SpriteY;
use crate::ppu::sprite::sprite_height::SpriteHeight;
use crate::ppu::tile_number::TileNumber;

use super::palette::bank_color_assigner::BankColorAssigner;
use super::sprite::sprite_evaluator::SpriteEvaluator;

pub struct Ppu {
    oam_registers: OamRegisters,
    oam_register_index: usize,
    sprite_evaluator: SpriteEvaluator,

    next_tile_number: TileNumber,
    pattern_register: PatternRegister,
    attribute_register: AttributeRegister,

    next_sprite_tile_number: TileNumber,
    current_sprite_y: SpriteY,
    sprite_visible: bool,

    frame_actions: FrameActions,

    pattern_source_frame: Frame,
    bank_color_assigner: BankColorAssigner,
}

impl Ppu {
    pub fn new(mem: &Memory) -> Ppu {
        Ppu {
            oam_registers: OamRegisters::new(),
            oam_register_index: 0,
            sprite_evaluator: SpriteEvaluator::new(),

            next_tile_number: TileNumber::new(0),
            pattern_register: PatternRegister::new(),
            attribute_register: AttributeRegister::new(),

            next_sprite_tile_number: TileNumber::new(0),
            current_sprite_y: SpriteY::new(0),
            sprite_visible: false,

            frame_actions: NTSC_FRAME_ACTIONS.clone(),

            pattern_source_frame: Frame::new(),
            bank_color_assigner: BankColorAssigner::new(mem),
        }
    }

    pub fn step(&mut self, mapper: &mut dyn Mapper, mem: &mut Memory, frame: &mut Frame) {
        let tick_result = mem.ppu_regs.tick();
        if tick_result.rendering_toggled == Some(Toggle::Disable) {
            // "... when rendering is disabled, the value on the PPU address bus is the current value of the v register."
            mapper.set_ppu_address_bus(mem, mem.ppu_regs.current_address);
        }

        let clock = *mem.ppu_regs.clock();
        if log_enabled!(target: "ppusteps", Info) {
            info!(" {clock}\t{}", self.frame_actions.format_current_cycle_actions(&clock));
        }

        // TODO: Figure out how to eliminate duplication and the index.
        let len = self.frame_actions.current_cycle_actions(&clock).len();
        for i in 0..len {
            let cycle_action = self.frame_actions.current_cycle_actions(&clock)[i];
            self.execute_cycle_action(mapper, mem, frame, cycle_action);
        }

        if mem.ppu_regs.can_generate_nmi() {
            mem.cpu_pinout.nmi_signal_detector.set_value(SignalLevel::Low);
        } else {
            mem.cpu_pinout.nmi_signal_detector.set_value(SignalLevel::High);
        }

        mapper.on_end_of_ppu_cycle();
    }

    fn execute_cycle_action(&mut self, mapper: &mut dyn Mapper, mem: &mut Memory, frame: &mut Frame, cycle_action: CycleAction) {
        use CycleAction::*;
        match cycle_action {
            SetPatternIndexAddress =>
                mapper.set_ppu_address_bus(mem, mem.ppu_regs.address_in_name_table()),
            SetPaletteIndexAddress =>
                mapper.set_ppu_address_bus(mem, mem.ppu_regs.address_in_attribute_table()),
            SetPatternLowAddress =>
                mapper.set_ppu_address_bus(mem, mem.ppu_regs.address_for_low_pattern_byte(self.next_tile_number)),
            SetPatternHighAddress =>
                mapper.set_ppu_address_bus(mem, mem.ppu_regs.address_for_high_pattern_byte(self.next_tile_number)),

            GetPatternIndex => self.next_tile_number = TileNumber::new(mapper.ppu_internal_read(mem).value()),
            GetPatternLowByte => {
                let pattern_low = mapper.ppu_internal_read(mem);
                if !mem.ppu_regs.background_enabled() && !mem.ppu_regs.sprites_enabled() { return; }
                self.pattern_register.set_pending_low_byte(pattern_low);
            }
            GetPatternHighByte => {
                let pattern_high = mapper.ppu_internal_read(mem);
                if !mem.ppu_regs.background_enabled() && !mem.ppu_regs.sprites_enabled() { return; }
                self.pattern_register.set_pending_high_byte(pattern_high);
            }
            GetPaletteIndex => {
                let attribute_byte = mapper.ppu_internal_read(mem).value();
                if !mem.ppu_regs.background_enabled() && !mem.ppu_regs.sprites_enabled() { return; }
                self.attribute_register.set_pending_palette_table_index(mem.ppu_regs.palette_table_index(attribute_byte));
            }
            PrepareForNextTile => {
                if !mem.ppu_regs.background_enabled() && !mem.ppu_regs.sprites_enabled() { return; }
                self.attribute_register.prepare_next_palette_table_index();
                self.pattern_register.load_next_palette_indexes();
            }
            PrepareForNextPixel => {
                if !mem.ppu_regs.background_enabled() && !mem.ppu_regs.sprites_enabled() { return; }
                self.pattern_register.shift_left();
                self.attribute_register.push_next_palette_table_index();
            }

            GotoNextTileColumn => {
                if !mem.ppu_regs.background_enabled() && !mem.ppu_regs.sprites_enabled() { return; }
                mem.ppu_regs.current_address.increment_coarse_x_scroll();
            }
            GotoNextPixelRow => {
                if !mem.ppu_regs.background_enabled() && !mem.ppu_regs.sprites_enabled() { return; }
                mem.ppu_regs.current_address.increment_fine_y_scroll();
            }
            ResetTileColumn => {
                if !mem.ppu_regs.background_enabled() && !mem.ppu_regs.sprites_enabled() { return; }
                mem.ppu_regs.reset_tile_column();
            }
            SetPixel => {
                let clock = *mem.ppu_regs.clock();
                let (pixel_column, pixel_row) = PixelIndex::try_from_clock(&clock).unwrap().to_column_row();
                if mem.ppu_regs.background_enabled() {
                    let palette_table = &mem.palette_table();
                    // TODO: Figure out where this goes. Maybe have frame call palette_table when displaying.
                    frame.set_universal_background_rgb(palette_table.universal_background_rgb());

                    let column_in_tile = mem.ppu_regs.fine_x_scroll;
                    let palette_table_index = self.attribute_register.palette_table_index(column_in_tile);
                    let palette = palette_table.background_palette(palette_table_index);

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

                if mem.ppu_regs.sprites_enabled() || mem.ppu_regs.background_enabled() {
                    let (mut sprite_pixel, priority, is_sprite_0, ppu_peek) = self.oam_registers.step(&mem.palette_table());
                    // HACK: Transparent sprites on row 0 should be a natural consequence of the shifter pipeline instead.
                    if pixel_row == PixelRow::ZERO {
                        sprite_pixel = Rgbt::Transparent;
                    }

                    if mem.ppu_regs.sprites_enabled() {
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
                        self.pattern_source_frame.set_sprite_pixel(pixel_column, pixel_row, bank_pixel, priority, false);
                    }
                }

                // https://wiki.nesdev.org/w/index.php?title=PPU_OAM#Sprite_zero_hits
                if mem.ppu_regs.sprites_enabled() && mem.ppu_regs.background_enabled()
                    && frame.pixel(mem.ppu_regs.mask(), pixel_column, pixel_row).1.hit()
                {
                    mem.ppu_regs.set_sprite0_hit();
                }
            }

            MaybeCorruptOamStart => {
                if !mem.ppu_regs.background_enabled() && !mem.ppu_regs.sprites_enabled() { return; }
                // Unclear if these are the correct cycles to trigger on.
                let oam_addr = mem.ppu_regs.oam_addr;
                let cycle = mem.ppu_regs.clock().cycle();
                mem.oam.maybe_corrupt_starting_byte(oam_addr, cycle);
            }

            ResetOamAddress => {
                if !mem.ppu_regs.background_enabled() && !mem.ppu_regs.sprites_enabled() { return; }
                mem.ppu_regs.oam_addr.reset();
            }

            StartClearingSecondaryOam => {
                info!(target: "ppustage", "{}\t\tCLEARING SECONDARY OAM", mem.ppu_regs.clock());
                self.sprite_evaluator.start_clearing_secondary_oam();
            }
            StartSpriteEvaluation => {
                info!(target: "ppustage", "\t\tSPRITE EVALUATION");
                self.sprite_evaluator.start_sprite_evaluation();
                self.oam_register_index = 0;
            }
            StartLoadingOamRegisters => {
                info!(target: "ppustage", "\t\tLoading OAM registers.");
                self.sprite_evaluator.start_loading_oam_registers();
                self.oam_registers.set_sprite_0_presence(self.sprite_evaluator.sprite_0_present());
            }
            StopLoadingOamRegisters => {
                info!(target: "ppustage", "\t\tLoading OAM registers ended.");
            }
            ReadOamByte => {
                if !mem.ppu_regs.background_enabled() && !mem.ppu_regs.sprites_enabled() { return; }
                self.sprite_evaluator.read_oam(mem);
            }
            WriteSecondaryOamByte => {
                if !mem.ppu_regs.background_enabled() && !mem.ppu_regs.sprites_enabled() { return; }
                self.sprite_evaluator.write_secondary_oam(mem);

            }
            ReadSpriteY => {
                if !mem.ppu_regs.background_enabled() && !mem.ppu_regs.sprites_enabled() { return; }
                self.current_sprite_y = SpriteY::new(self.sprite_evaluator.read_secondary_oam_and_advance());
            }
            ReadSpritePatternIndex => {
                if !mem.ppu_regs.background_enabled() && !mem.ppu_regs.sprites_enabled() { return; }
                self.next_sprite_tile_number = TileNumber::new(self.sprite_evaluator.read_secondary_oam_and_advance());
            }
            ReadSpriteAttributes => {
                if !mem.ppu_regs.background_enabled() && !mem.ppu_regs.sprites_enabled() { return; }
                let attributes = SpriteAttributes::from_u8(self.sprite_evaluator.read_secondary_oam_and_advance());
                self.oam_registers.registers[self.oam_register_index].set_attributes(attributes);
            }
            ReadSpriteX => {
                if !mem.ppu_regs.background_enabled() && !mem.ppu_regs.sprites_enabled() { return; }
                let x_counter = self.sprite_evaluator.read_secondary_oam_and_advance();
                self.oam_registers.registers[self.oam_register_index].set_x_counter(x_counter);
            }
            DummyReadSpriteX => {
                // TODO
            }

            SetSpritePatternLowAddress => {
                let select_high = false;
                let addr;
                (addr, self.sprite_visible) = self.current_sprite_pattern_address(mem, select_high);
                mapper.set_ppu_address_bus(mem, addr);
            }
            SetSpritePatternHighAddress => {
                let select_high = true;
                let addr;
                (addr, self.sprite_visible) = self.current_sprite_pattern_address(mem, select_high);
                mapper.set_ppu_address_bus(mem, addr);
            }
            GetSpritePatternLowByte => {
                let pattern_low = mapper.ppu_internal_read(mem);
                if (mem.ppu_regs.background_enabled() || mem.ppu_regs.sprites_enabled()) && self.sprite_visible {
                    self.oam_registers.registers[self.oam_register_index].set_pattern_low(pattern_low);
                }
            }
            GetSpritePatternHighByte => {
                let pattern_high = mapper.ppu_internal_read(mem);
                if (mem.ppu_regs.background_enabled() || mem.ppu_regs.sprites_enabled()) && self.sprite_visible {
                    self.oam_registers.registers[self.oam_register_index].set_pattern_high(pattern_high);
                }
            }
            IncrementOamRegisterIndex => {
                self.oam_register_index += 1;
            }

            // TODO: Remove this section in favor of using EdgeDetectors.
            StartVisibleScanlines => {
                info!(target: "ppustage", "{}\tVISIBLE SCANLINES", mem.ppu_regs.clock());
            }
            StartPostRenderScanline => {
                info!(target: "ppustage", "{}\tPOST-RENDER SCANLINE", mem.ppu_regs.clock());
            }
            StartVblankScanlines => {
                info!(target: "ppustage", "{}\tVBLANK SCANLINES", mem.ppu_regs.clock());
            }
            StartPreRenderScanline => {
                info!(target: "ppustage", "{}\tPRE-RENDER SCANLINE", mem.ppu_regs.clock());
            }
            StartReadingBackgroundTiles => {
                info!(target: "ppustage", "{}\t\tREADING BACKGROUND TILES", mem.ppu_regs.clock());
            }
            StopReadingBackgroundTiles => {
                info!(target: "ppustage", "{}\t\tENDED READING BACKGROUND TILES", mem.ppu_regs.clock());
            }

            StartVblank => {
                if mem.ppu_regs.suppress_vblank_active {
                    info!(target: "ppuflags", " {}\tSuppressing vblank.", mem.ppu_regs.clock());
                } else {
                    let clock = *mem.ppu_regs.clock();
                    mem.ppu_regs.start_vblank(&clock);
                    // "During VBlank ... the value on the PPU address bus is the current value of the v register."
                    mapper.set_ppu_address_bus(mem, mem.ppu_regs.current_address);
                }

                mem.ppu_regs.suppress_vblank_active = false;
            }
            SetInitialScrollOffsets => {
                if !mem.ppu_regs.background_enabled() { return; }
                mem.ppu_regs.current_address = mem.ppu_regs.next_address;
            }
            SetInitialYScroll => {
                if !mem.ppu_regs.background_enabled() { return; }
                let next_address = mem.ppu_regs.next_address;
                mem.ppu_regs.current_address.copy_y_scroll(next_address);
            }

            ClearFlags => {
                let clock = *mem.ppu_regs.clock();
                mem.ppu_regs.stop_vblank(&clock);
                mem.ppu_regs.clear_sprite0_hit();
                mem.ppu_regs.clear_sprite_overflow();
                mem.ppu_regs.clear_reset();

            }
        }
    }

    pub fn pattern_source_frame(&self) -> &Frame {
        &self.pattern_source_frame
    }

    fn current_sprite_pattern_address(&self, mem: &Memory, select_high: bool) -> (PpuAddress, bool) {
        let sprite_table_side = mem.ppu_regs.sprite_table_side();
        let sprite_height = mem.ppu_regs.sprite_height();
        let sprite_table_side = match sprite_height {
            SpriteHeight::Normal => sprite_table_side,
            SpriteHeight::Tall => self.next_sprite_tile_number.tall_sprite_pattern_table_side(),
        };

        let address;
        let visible;
        if let Some(pixel_row) = mem.ppu_regs.clock().scanline_pixel_row() {
            let attributes = self.oam_registers.registers[self.oam_register_index].attributes();
            if let Some((tile_number, row_in_half, v)) = self.next_sprite_tile_number.number_and_row(
                self.current_sprite_y,
                attributes.flip_vertically(),
                sprite_height,
                pixel_row
            ) {
                visible = v;
                address = PpuAddress::in_pattern_table(
                    sprite_table_side, tile_number, row_in_half, select_high);
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
