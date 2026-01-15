use log::{info, log_enabled};
use log::Level::Info;

use crate::mapper::Mapper;
use crate::bus::Bus;
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
    pub fn new(bank_color_assigner: BankColorAssigner) -> Ppu {
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
            bank_color_assigner,
        }
    }

    pub fn step(bus: &mut Bus, mapper: &mut dyn Mapper, frame: &mut Frame) {
        let tick_result = bus.ppu_regs.tick(bus.master_clock.ppu_clock());
        if tick_result.rendering_toggled == Some(Toggle::Disable) {
            // "... when rendering is disabled, the value on the PPU address bus is the current value of the v register."
            bus.set_ppu_address_bus(mapper, bus.ppu_regs.current_address);
        }

        if log_enabled!(target: "ppusteps", Info) {
            info!(" {}\t{}", bus.ppu_clock(), bus.ppu.frame_actions.format_current_cycle_actions(bus.ppu_clock()));
        }

        // TODO: Figure out how to eliminate duplication and the index.
        let len = bus.ppu.frame_actions.current_cycle_actions(bus.ppu_clock()).len();
        for i in 0..len {
            let cycle_action = bus.ppu.frame_actions.current_cycle_actions(bus.master_clock.ppu_clock())[i];
            Ppu::execute_cycle_action(bus, mapper, frame, cycle_action);
        }

        if bus.ppu_regs.vblank_active && bus.ppu_regs.nmi_enabled() {
            bus.cpu_pinout.nmi_signal_detector.set_value(SignalLevel::Low);
        } else {
            bus.cpu_pinout.nmi_signal_detector.set_value(SignalLevel::High);
        }

        mapper.on_end_of_ppu_cycle();
    }

    fn execute_cycle_action(bus: &mut Bus, mapper: &mut dyn Mapper, frame: &mut Frame, cycle_action: CycleAction) {
        use CycleAction::*;
        match cycle_action {
            SetPatternIndexAddress =>
                bus.set_ppu_address_bus(mapper, bus.ppu_regs.current_address.to_name_table_address()),
            SetPaletteIndexAddress =>
                bus.set_ppu_address_bus(mapper, bus.ppu_regs.current_address.to_attribute_table_address()),
            SetPatternLowAddress => {
                let addr = PpuAddress::in_pattern_table(
                    bus.ppu_regs.background_table_side(),
                    bus.ppu.next_tile_number,
                    bus.ppu_regs.current_address.fine_y_scroll(),
                    false,
                );
                bus.set_ppu_address_bus(mapper, addr);
            }
            SetPatternHighAddress => {
                let addr = PpuAddress::in_pattern_table(
                    bus.ppu_regs.background_table_side(),
                    bus.ppu.next_tile_number,
                    bus.ppu_regs.current_address.fine_y_scroll(),
                    true,
                );
                bus.set_ppu_address_bus(mapper, addr);
            }

            GetPatternIndex => bus.ppu.next_tile_number = TileNumber::new(bus.ppu_internal_read(mapper).value()),
            GetPatternLowByte => {
                let pattern_low = bus.ppu_internal_read(mapper);
                if !bus.ppu_regs.background_enabled() && !bus.ppu_regs.sprites_enabled() { return; }
                bus.ppu.pattern_register.set_pending_low_byte(pattern_low);
            }
            GetPatternHighByte => {
                let pattern_high = bus.ppu_internal_read(mapper);
                if !bus.ppu_regs.background_enabled() && !bus.ppu_regs.sprites_enabled() { return; }
                bus.ppu.pattern_register.set_pending_high_byte(pattern_high);
            }
            GetPaletteIndex => {
                let attribute_byte = bus.ppu_internal_read(mapper).value();
                if !bus.ppu_regs.background_enabled() && !bus.ppu_regs.sprites_enabled() { return; }
                let index = bus.ppu_regs.current_address.to_palette_table_index(attribute_byte);
                bus.ppu.attribute_register.set_pending_palette_table_index(index);
            }
            PrepareForNextTile => {
                if !bus.ppu_regs.background_enabled() && !bus.ppu_regs.sprites_enabled() { return; }
                bus.ppu.attribute_register.prepare_next_palette_table_index();
                bus.ppu.pattern_register.load_next_palette_indexes();
            }
            PrepareForNextPixel => {
                if !bus.ppu_regs.background_enabled() && !bus.ppu_regs.sprites_enabled() { return; }
                bus.ppu.pattern_register.shift_left();
                bus.ppu.attribute_register.push_next_palette_table_index();
            }

            GotoNextTileColumn => {
                if !bus.ppu_regs.background_enabled() && !bus.ppu_regs.sprites_enabled() { return; }
                bus.ppu_regs.current_address.increment_coarse_x_scroll();
            }
            GotoNextPixelRow => {
                if !bus.ppu_regs.background_enabled() && !bus.ppu_regs.sprites_enabled() { return; }
                bus.ppu_regs.current_address.increment_fine_y_scroll();
            }
            ResetTileColumn => {
                if !bus.ppu_regs.background_enabled() && !bus.ppu_regs.sprites_enabled() { return; }
                bus.ppu_regs.current_address.set_tile_column_from(bus.ppu_regs.next_address);
            }
            SetPixel => {
                let clock = bus.ppu_clock();
                let (pixel_column, pixel_row) = PixelIndex::try_from_clock(clock).unwrap().to_column_row();
                if bus.ppu_regs.background_enabled() {
                    let palette_table = &bus.palette_table();
                    // TODO: Figure out where this goes. Maybe have frame call palette_table when displaying.
                    frame.set_universal_background_rgb(palette_table.universal_background_rgb());

                    let column_in_tile = bus.ppu_regs.fine_x_scroll;
                    let palette_table_index = bus.ppu.attribute_register.palette_table_index(column_in_tile);
                    let palette = palette_table.background_palette(palette_table_index);

                    let background_pixel = bus.ppu.pattern_register
                        .palette_index(column_in_tile)
                        .map_or(Rgbt::Transparent, |palette_index| Rgbt::Opaque(palette[palette_index]));

                    frame.set_background_pixel(pixel_column, pixel_row, background_pixel);

                    let bank_pixel = if background_pixel.is_transparent() {
                        Rgbt::Transparent
                    } else {
                        let rgb = bus.ppu.bank_color_assigner.rgb_for_source(bus.ppu.pattern_register.current_peek().source());
                        Rgbt::Opaque(rgb)
                    };
                    bus.ppu.pattern_source_frame.set_background_pixel(pixel_column, pixel_row, bank_pixel);
                }

                if bus.ppu_regs.sprites_enabled() || bus.ppu_regs.background_enabled() {
                    let (mut sprite_pixel, priority, is_sprite_0, ppu_peek) = bus.ppu.oam_registers.step(&bus.palette_table());
                    // HACK: Transparent sprites on row 0 should be a natural consequence of the shifter pipeline instead.
                    if pixel_row == PixelRow::ZERO {
                        sprite_pixel = Rgbt::Transparent;
                    }

                    if bus.ppu_regs.sprites_enabled() {
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
                            let rgb = bus.ppu.bank_color_assigner.rgb_for_source(ppu_peek.source());
                            Rgbt::Opaque(rgb)
                        };
                        bus.ppu.pattern_source_frame.set_sprite_pixel(pixel_column, pixel_row, bank_pixel, priority, false);
                    }
                }

                // https://wiki.nesdev.org/w/index.php?title=PPU_OAM#Sprite_zero_hits
                if bus.ppu_regs.sprites_enabled() && bus.ppu_regs.background_enabled()
                    && frame.pixel(bus.ppu_regs.mask(), pixel_column, pixel_row).1.hit()
                {
                    bus.ppu_regs.sprite0_hit = true;
                }
            }

            MaybeCorruptOamStart => {
                if !bus.ppu_regs.background_enabled() && !bus.ppu_regs.sprites_enabled() { return; }
                // Unclear if these are the correct cycles to trigger on.
                let oam_addr = bus.ppu_regs.oam_addr;
                let cycle = bus.ppu_clock().cycle();
                bus.oam.maybe_corrupt_starting_byte(oam_addr, cycle);
            }

            ResetOamAddress => {
                if !bus.ppu_regs.background_enabled() && !bus.ppu_regs.sprites_enabled() { return; }
                bus.ppu_regs.oam_addr.reset();
            }

            StartClearingSecondaryOam => {
                info!(target: "ppustage", "{}\t\tCLEARING SECONDARY OAM", bus.ppu_clock());
                bus.ppu.sprite_evaluator.start_clearing_secondary_oam();
            }
            StartSpriteEvaluation => {
                info!(target: "ppustage", "\t\tSPRITE EVALUATION");
                bus.ppu.sprite_evaluator.start_sprite_evaluation();
                bus.ppu.oam_register_index = 0;
            }
            StartLoadingOamRegisters => {
                info!(target: "ppustage", "\t\tLoading OAM registers.");
                bus.ppu.sprite_evaluator.start_loading_oam_registers();
                bus.ppu.oam_registers.set_sprite_0_presence(bus.ppu.sprite_evaluator.sprite_0_present());
            }
            StopLoadingOamRegisters => {
                info!(target: "ppustage", "\t\tLoading OAM registers ended.");
            }
            ReadOamByte => {
                if !bus.ppu_regs.background_enabled() && !bus.ppu_regs.sprites_enabled() { return; }
                bus.ppu.sprite_evaluator.read_oam(&bus.oam, &bus.ppu_regs);
            }
            WriteSecondaryOamByte => {
                if !bus.ppu_regs.background_enabled() && !bus.ppu_regs.sprites_enabled() { return; }
                bus.ppu.sprite_evaluator.write_secondary_oam(bus.master_clock.ppu_clock(), &mut bus.ppu_regs);
            }
            ReadSpriteY => {
                if !bus.ppu_regs.background_enabled() && !bus.ppu_regs.sprites_enabled() { return; }
                bus.ppu.current_sprite_y = SpriteY::new(bus.ppu.sprite_evaluator.read_secondary_oam_and_advance());
            }
            ReadSpritePatternIndex => {
                if !bus.ppu_regs.background_enabled() && !bus.ppu_regs.sprites_enabled() { return; }
                bus.ppu.next_sprite_tile_number = TileNumber::new(bus.ppu.sprite_evaluator.read_secondary_oam_and_advance());
            }
            ReadSpriteAttributes => {
                if !bus.ppu_regs.background_enabled() && !bus.ppu_regs.sprites_enabled() { return; }
                let attributes = SpriteAttributes::from_u8(bus.ppu.sprite_evaluator.read_secondary_oam_and_advance());
                bus.ppu.oam_registers.registers[bus.ppu.oam_register_index].set_attributes(attributes);
            }
            ReadSpriteX => {
                if !bus.ppu_regs.background_enabled() && !bus.ppu_regs.sprites_enabled() { return; }
                let x_counter = bus.ppu.sprite_evaluator.read_secondary_oam_and_advance();
                bus.ppu.oam_registers.registers[bus.ppu.oam_register_index].set_x_counter(x_counter);
            }
            DummyReadSpriteX => {
                // TODO
            }

            SetSpritePatternLowAddress => {
                let select_high = false;
                let addr;
                (addr, bus.ppu.sprite_visible) = bus.ppu.current_sprite_pattern_address(bus, select_high);
                bus.set_ppu_address_bus(mapper, addr);
            }
            SetSpritePatternHighAddress => {
                let select_high = true;
                let addr;
                (addr, bus.ppu.sprite_visible) = bus.ppu.current_sprite_pattern_address(bus, select_high);
                bus.set_ppu_address_bus(mapper, addr);
            }
            GetSpritePatternLowByte => {
                let pattern_low = bus.ppu_internal_read(mapper);
                if (bus.ppu_regs.background_enabled() || bus.ppu_regs.sprites_enabled()) && bus.ppu.sprite_visible {
                    bus.ppu.oam_registers.registers[bus.ppu.oam_register_index].set_pattern_low(pattern_low);
                }
            }
            GetSpritePatternHighByte => {
                let pattern_high = bus.ppu_internal_read(mapper);
                if (bus.ppu_regs.background_enabled() || bus.ppu_regs.sprites_enabled()) && bus.ppu.sprite_visible {
                    bus.ppu.oam_registers.registers[bus.ppu.oam_register_index].set_pattern_high(pattern_high);
                }
            }
            IncrementOamRegisterIndex => {
                bus.ppu.oam_register_index += 1;
            }

            // TODO: Remove this section in favor of using EdgeDetectors.
            StartVisibleScanlines => {
                info!(target: "ppustage", "{}\tVISIBLE SCANLINES", bus.ppu_clock());
            }
            StartPostRenderScanline => {
                info!(target: "ppustage", "{}\tPOST-RENDER SCANLINE", bus.ppu_clock());
            }
            StartVblankScanlines => {
                info!(target: "ppustage", "{}\tVBLANK SCANLINES", bus.ppu_clock());
            }
            StartPreRenderScanline => {
                info!(target: "ppustage", "{}\tPRE-RENDER SCANLINE", bus.ppu_clock());
            }
            StartReadingBackgroundTiles => {
                info!(target: "ppustage", "{}\t\tREADING BACKGROUND TILES", bus.ppu_clock());
            }
            StopReadingBackgroundTiles => {
                info!(target: "ppustage", "{}\t\tENDED READING BACKGROUND TILES", bus.ppu_clock());
            }

            StartVblank => {
                if bus.ppu_regs.suppress_vblank_active {
                    info!(target: "ppuflags", " {}\tSuppressing vblank.", bus.ppu_clock());
                } else {
                    bus.ppu_regs.start_vblank(bus.master_clock.ppu_clock());
                    // "During VBlank ... the value on the PPU address bus is the current value of the v register."
                    bus.set_ppu_address_bus(mapper, bus.ppu_regs.current_address);
                }

                bus.ppu_regs.suppress_vblank_active = false;
            }
            SetInitialScrollOffsets => {
                if !bus.ppu_regs.background_enabled() { return; }
                bus.ppu_regs.current_address = bus.ppu_regs.next_address;
            }
            SetInitialYScroll => {
                if !bus.ppu_regs.background_enabled() { return; }
                let next_address = bus.ppu_regs.next_address;
                bus.ppu_regs.current_address.copy_y_scroll(next_address);
            }

            ClearFlags => {
                bus.ppu_regs.stop_vblank(bus.master_clock.ppu_clock());
                bus.ppu_regs.sprite0_hit = false;
                bus.ppu_regs.sprite_overflow = false;

                // At startup, CIRAM writes are disabled until the PPU has been running for a while.
                // TODO: Determine the correct place to call this.
                bus.ciram.enable_writes();
            }
        }
    }

    pub fn pattern_source_frame(&self) -> &Frame {
        &self.pattern_source_frame
    }

    fn current_sprite_pattern_address(&self, bus: &Bus, select_high: bool) -> (PpuAddress, bool) {
        let sprite_table_side = bus.ppu_regs.sprite_table_side();
        let sprite_height = bus.ppu_regs.sprite_height();
        let sprite_table_side = match sprite_height {
            SpriteHeight::Normal => sprite_table_side,
            SpriteHeight::Tall => self.next_sprite_tile_number.tall_sprite_pattern_table_side(),
        };

        let address;
        let visible;
        if let Some(pixel_row) = bus.ppu_clock().scanline_pixel_row() {
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
