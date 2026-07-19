use log::{info, log_enabled};
use log::Level::Info;
use ux::u5;

use crate::mapper::mapper::Mapper;
use crate::bus::Bus;
use crate::memory::ppu::chr_memory::PpuPeek;
use crate::memory::ppu::ppu_address::{PaletteRamIndex, PpuAddress};
use crate::memory::signal_level::SignalLevel;
use crate::ppu::cycle_action::cycle_action::CycleAction;
use crate::ppu::cycle_action::frame_actions::{FrameActions, NTSC_FRAME_ACTIONS};
use crate::ppu::palette::color::Color;
use crate::ppu::palette::rgb::Rgb;
use crate::ppu::palette::rgbt::Rgbt;
use crate::ppu::palette::color_t::ColorT;
use crate::ppu::pattern_table_side::PatternTableSide;
use crate::ppu::pixel_index::{PixelColumn, PixelIndex, PixelRow};
use crate::ppu::register::ppu_registers::Toggle;
use crate::ppu::register::registers::attribute_register::AttributeRegister;
use crate::ppu::register::registers::pattern_register::PatternRegister;
use crate::ppu::render::frame::{DebugBuffer, FrameBuffer, Frame};
use crate::ppu::sprite::sprite_attributes::{Priority, SpriteAttributes};
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
    next_rendering_field_to_set: Option<(RenderingRegisterField, u8)>,
    next_register_value: PpuPeek,
    pending_register_shift: bool,

    next_sprite_tile_number: TileNumber,
    current_sprite_y: SpriteY,
    sprite_visible: bool,

    frame_actions: FrameActions,

    // Used only for debug screens
    background_buffer: FrameBuffer<ColorT>,
    sprite_buffer: FrameBuffer<ColorT>,
    pattern_source_debug_buffer: DebugBuffer<{PixelColumn::COLUMN_COUNT}, {PixelRow::ROW_COUNT}>,
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
            next_rendering_field_to_set: None,
            next_register_value: PpuPeek::VOID,
            pending_register_shift: false,

            next_sprite_tile_number: TileNumber::new(0),
            current_sprite_y: SpriteY::new(0),
            sprite_visible: false,

            frame_actions: NTSC_FRAME_ACTIONS.clone(),

            background_buffer: FrameBuffer::default(),
            sprite_buffer: FrameBuffer::default(),
            pattern_source_debug_buffer: DebugBuffer::new(Rgb::BLACK),
            bank_color_assigner,
        }
    }

    pub fn step_first_half(bus: &mut Bus, mapper: &mut dyn Mapper, frame: &mut Frame) {
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

        if bus.ppu_regs.suppress_vblank_active {
            bus.ppu_regs.vblank_active = false;
            bus.ppu_regs.suppress_vblank_active = false;
        }

        if bus.ppu_regs.vblank_active && bus.ppu_regs.nmi_enabled() {
            bus.cpu_pinout.nmi_signal_detector.set_value(SignalLevel::Low);
        } else {
            bus.cpu_pinout.nmi_signal_detector.set_value(SignalLevel::High);
        }

        mapper.on_end_of_ppu_cycle();
    }

    pub fn step_second_half(Bus { ppu, ppu_regs, .. }: &mut Bus) {
        if ppu.pending_register_shift && (ppu_regs.background_enabled() || ppu_regs.sprites_enabled()) {
            ppu.pending_register_shift = false;
            ppu.pattern_register.shift_left();
            ppu.attribute_register.push_next_palette_table_index();
        }

        let Some((next_rendering_field_to_set, cycles_remaining)) = ppu.next_rendering_field_to_set.take() else {
            return;
        };

        if cycles_remaining > 1 {
            ppu.next_rendering_field_to_set = Some((next_rendering_field_to_set, cycles_remaining - 1));
            return;
        }

        match next_rendering_field_to_set {
            RenderingRegisterField::PatternIndex => {
                ppu.next_tile_number = TileNumber::new(ppu.next_register_value.value());
            }
            RenderingRegisterField::PaletteIndex => {
                let index = ppu_regs.current_address.to_palette_table_index(ppu.next_register_value.value());
                ppu.attribute_register.set_pending_palette_table_index(index);
            }
            RenderingRegisterField::BackgroundPatternLow => {
                ppu.pattern_register.set_pending_low_byte(ppu.next_register_value);
            }
            RenderingRegisterField::BackgroundPatternHighAndNextTile => {
                ppu.pattern_register.set_pending_high_byte(ppu.next_register_value);
                ppu.attribute_register.prepare_next_palette_table_index();
                ppu.pattern_register.load_next_palette_indexes();
                ppu_regs.current_address.increment_coarse_x_scroll();
            }
            RenderingRegisterField::SpritePatternLow => {
                // TODO: Determine if this check should happen at the same time as rendering_enabled()
                if ppu.sprite_visible {
                    ppu.oam_registers[ppu.oam_register_index].set_pattern_low(ppu.next_register_value);
                }
            }
            RenderingRegisterField::SpritePatternHighAndNextSprite => {
                // TODO: Determine if this check should happen at the same time as rendering_enabled()
                if ppu.sprite_visible {
                    ppu.oam_registers[ppu.oam_register_index].set_pattern_high(ppu.next_register_value);
                }

                ppu.oam_register_index += 1;
            }
        }
    }

    fn execute_cycle_action(bus: &mut Bus, mapper: &mut dyn Mapper, frame: &mut Frame, cycle_action: CycleAction) {
        use CycleAction::*;
        match cycle_action {
            SetPatternIndexAddress => {
                bus.set_ppu_address_bus(mapper, bus.ppu_regs.current_address.to_name_table_address());
                bus.ppu.next_rendering_field_to_set = Some((RenderingRegisterField::PatternIndex, 2));
            }
            SetPaletteIndexAddress => {
                bus.set_ppu_address_bus(mapper, bus.ppu_regs.current_address.to_attribute_table_address());
                if bus.ppu_regs.rendering_enabled() {
                    bus.ppu.next_rendering_field_to_set = Some((RenderingRegisterField::PaletteIndex, 2));
                }
            }
            SetPatternLowAddress => {
                let addr = PpuAddress::in_pattern_table(
                    bus.ppu_regs.background_table_side(),
                    bus.ppu.next_tile_number,
                    bus.ppu_regs.current_address.fine_y_scroll(),
                    false,
                );
                bus.set_ppu_address_bus(mapper, addr);
                if bus.ppu_regs.rendering_enabled() {
                    bus.ppu.next_rendering_field_to_set = Some((RenderingRegisterField::BackgroundPatternLow, 2));
                }
            }
            SetPatternHighAddress => {
                let addr = PpuAddress::in_pattern_table(
                    bus.ppu_regs.background_table_side(),
                    bus.ppu.next_tile_number,
                    bus.ppu_regs.current_address.fine_y_scroll(),
                    true,
                );
                bus.set_ppu_address_bus(mapper, addr);
                if bus.ppu_regs.rendering_enabled() {
                    bus.ppu.next_rendering_field_to_set = Some((RenderingRegisterField::BackgroundPatternHighAndNextTile, 2));
                }
            }

            Read => {
                bus.ppu.next_register_value = bus.ppu_read(mapper);
            }
            PrepareForNextPixel => {
                bus.ppu.pending_register_shift = true;
            }

            GotoNextPixelRow => {
                if !bus.ppu_regs.rendering_enabled() { return; }
                bus.ppu_regs.current_address.increment_fine_y_scroll();
            }
            ResetTileColumn => {
                if !bus.ppu_regs.rendering_enabled() { return; }
                bus.ppu_regs.current_address.set_tile_column_from(bus.ppu_regs.next_address);
            }
            SetPixel => {
                let clock = bus.ppu_clock();
                let pixel_index = PixelIndex::try_from_clock(clock).unwrap();
                let ppumask = bus.ppu_regs.mask();

                let mut background_color = ColorT::Transparent;
                let mut background_bank_pixel = None;
                if bus.ppu_regs.background_enabled() {
                    let column_in_tile = bus.ppu_regs.fine_x_scroll;
                    let palette_table_index = bus.ppu.attribute_register.palette_table_index(column_in_tile);
                    let palette = bus.palette_ram().background_palette(palette_table_index);

                    background_color = bus.ppu.pattern_register
                        .palette_index(column_in_tile)
                        .map_or(ColorT::Transparent, |palette_index| ColorT::Opaque(palette[palette_index]));
                    background_bank_pixel = Some(if background_color.is_transparent() {
                        Rgbt::Transparent
                    } else {
                        let rgb = bus.ppu.bank_color_assigner.rgb_for_source(bus.ppu.pattern_register.current_peek().source());
                        Rgbt::Opaque(rgb)
                    });
                }

                // This is not delayed, unlike ppu_regs.rendering_enabled()
                let rendering_enabled = bus.ppu_regs.background_enabled() || bus.ppu_regs.sprites_enabled();
                let (mut sprite_color, sprite_priority, is_sprite_0, ppu_peek) =
                    bus.ppu.oam_registers.step(&bus.palette_ram, rendering_enabled);
                let mut color;
                if rendering_enabled {
                    if !bus.ppu_regs.sprites_enabled() {
                        sprite_color = ColorT::Transparent;
                    }

                    let PixelIndex { column, row } = pixel_index;

                    use ColorT::{Opaque, Transparent};
                    if !ppumask.left_background_columns_enabled() && column.is_in_left_margin() {
                        background_color = Transparent;
                    }

                    if !ppumask.left_sprite_columns_enabled() && column.is_in_left_margin() {
                        sprite_color = Transparent;
                    }

                    color = match (background_color, sprite_color, sprite_priority) {
                        (Transparent  , Transparent  , _) => bus.palette_ram.backdrop_color(),
                        (Transparent  , Opaque(color), _) => color,
                        (Opaque(color), Transparent  , _) => color,
                        (Opaque(_)    , Opaque(color), Priority::InFront) => color,
                        (Opaque(color), Opaque(_)    , Priority::Behind ) => color,
                    };

                    // These two are just for debug screens.
                    bus.ppu.background_buffer[pixel_index] = background_color;
                    bus.ppu.sprite_buffer[pixel_index] = sprite_color;

                    // https://wiki.nesdev.org/w/index.php?title=PPU_OAM#Sprite_zero_hits
                    let sprite_0_hit =
                        is_sprite_0 &&
                        column < PixelColumn::MAX &&
                        row <= PixelRow::MAX &&
                        background_color.is_opaque() &&
                        sprite_color.is_opaque();
                    if sprite_0_hit {
                        bus.ppu_regs.sprite0_hit_pending = true;
                    }

                    let sprite_bank_pixel = Some(if sprite_color.is_transparent() {
                        Rgbt::Transparent
                    } else {
                        let rgb = bus.ppu.bank_color_assigner.rgb_for_source(ppu_peek.source());
                        Rgbt::Opaque(rgb)
                    });
                    let is_sprite_pixel = background_color.is_transparent()
                        || (sprite_color.is_opaque() && sprite_priority == Priority::InFront);
                    let bank_pixel = if is_sprite_pixel { sprite_bank_pixel } else { background_bank_pixel };
                    if let Some(bank_pixel) = bank_pixel {
                        let column = pixel_index.column.to_usize();
                        let row = pixel_index.row.to_usize();
                        bus.ppu.pattern_source_debug_buffer.write_rgbt(column, row, bank_pixel);
                    }
                } else {
                    color = if bus.ppu_regs.current_address.is_in_palette_table() {
                        let index = PaletteRamIndex::new(u5::new((bus.ppu_regs.current_address.to_u16() & 0b0001_1111) as u8));
                        Color::from(bus.palette_ram.peek(&bus.ppu_regs, index).value())
                    } else {
                        bus.palette_ram.backdrop_color()
                    };
                }

                if !bus.composite_decoders.show_overscan && pixel_index.is_in_overscan_region() {
                    color = Color::BLACK;
                }

                bus.composite_decoders.get_mut().set_color(frame, &bus.master_clock, color, ppumask.emphasis());
            }

            MaybeCorruptOamStart => {
                if !bus.ppu_regs.rendering_enabled() { return; }
                // Unclear if these are the correct cycles to trigger on.
                let oam_addr = bus.ppu_regs.oam_addr;
                bus.oam.maybe_corrupt_starting_byte(bus.master_clock.ppu_clock(), oam_addr, bus.ppu_regs.rendering_enabled());
            }

            ResetOamAddress => {
                if !bus.ppu_regs.background_enabled() && !bus.ppu_regs.sprites_enabled() { return; }
                bus.ppu_regs.oam_addr.reset();
            }

            StartClearingSecondaryOam => {
                info!(target: "ppustage", "{}\t\tCLEARING SECONDARY OAM", bus.ppu_clock());
                bus.ppu.sprite_evaluator.start_clearing_secondary_oam();
            }
            ClearOamRegisterIndex => {
                bus.ppu.oam_register_index = 0;
            }
            StartSpriteEvaluation => {
                info!(target: "ppustage", "\t\tSPRITE EVALUATION");
                bus.ppu.sprite_evaluator.start_sprite_evaluation();
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
                if !bus.ppu_regs.rendering_enabled() { return; }
                bus.ppu.sprite_evaluator.read_oam(&mut bus.oam, &mut bus.oam_latch, &bus.master_clock.ppu_clock(), &bus.ppu_regs);
            }
            WriteSecondaryOamByte => {
                if !bus.ppu_regs.rendering_enabled() { return; }
                bus.ppu.sprite_evaluator.write_secondary_oam(&mut bus.oam_latch, bus.master_clock.ppu_clock(), &mut bus.ppu_regs);
            }
            ReadSpriteY => {
                if !bus.ppu_regs.rendering_enabled() { return; }
                bus.ppu.current_sprite_y = SpriteY::new(bus.ppu.sprite_evaluator.read_secondary_oam_and_advance());
            }
            ReadSpritePatternIndex => {
                if !bus.ppu_regs.rendering_enabled() { return; }
                bus.ppu.next_sprite_tile_number = TileNumber::new(bus.ppu.sprite_evaluator.read_secondary_oam_and_advance());
            }
            ReadSpriteAttributes => {
                if !bus.ppu_regs.rendering_enabled() { return; }
                let attributes = SpriteAttributes::from_u8(bus.ppu.sprite_evaluator.read_secondary_oam_and_advance());
                bus.ppu.oam_registers[bus.ppu.oam_register_index].set_attributes(attributes);
            }
            ReadSpriteX => {
                if !bus.ppu_regs.rendering_enabled() { return; }
                let x_counter = bus.ppu.sprite_evaluator.read_secondary_oam_and_advance();
                bus.ppu.oam_registers[bus.ppu.oam_register_index].set_x_counter(x_counter);
            }
            DummyReadSpriteX => {
                // TODO
            }
            MaybeClearSpriteX => {
                if !bus.ppu_regs.rendering_enabled() {
                    // This is a quirk of the rendering pipeline. There may be a better way to represent this.
                    for i in 0..8 {
                        bus.ppu.oam_registers[i].set_x_counter(0);
                    }
                }
            }

            SetSpritePatternLowAddress => {
                let select_high = false;
                let addr;
                (addr, bus.ppu.sprite_visible) = bus.ppu.current_sprite_pattern_address(bus, select_high);
                bus.set_ppu_address_bus(mapper, addr);

                if bus.ppu_regs.rendering_enabled() {
                    bus.ppu.next_rendering_field_to_set = Some((RenderingRegisterField::SpritePatternLow, 2));
                }
            }
            SetSpritePatternHighAddress => {
                let select_high = true;
                let addr;
                (addr, bus.ppu.sprite_visible) = bus.ppu.current_sprite_pattern_address(bus, select_high);
                bus.set_ppu_address_bus(mapper, addr);

                if bus.ppu_regs.rendering_enabled() {
                    bus.ppu.next_rendering_field_to_set = Some((RenderingRegisterField::SpritePatternHighAndNextSprite, 2));
                }
            }

            // TODO: Remove this section in favor of using EdgeDetectors.
            StartVisibleScanlines => {
                info!(target: "ppustage", "{}\tVISIBLE SCANLINES", bus.ppu_clock());
            }
            StartPostRenderScanline => {
                info!(target: "ppustage", "{}\tPOST-RENDER SCANLINE", bus.ppu_clock());
                // Move OAM towards its decayed state once per frame. Placing this here is arbitrary.
                bus.oam.maybe_decay();
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
                bus.ppu_regs.vblank_active = true;
                // "During VBlank ... the value on the PPU address bus is the current value of the v register."
                bus.set_ppu_address_bus(mapper, bus.ppu_regs.current_address);
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
                bus.ppu_regs.vblank_active = false;
                bus.ppu_regs.sprite0_hit = false;
                bus.ppu_regs.sprite_overflow = false;

                // At startup, CIRAM writes are disabled until the PPU has been running for a while.
                // TODO: Determine the correct place to call this.
                bus.ciram.enable_writes();
            }
        }
    }

    pub fn background_buffer(&self) -> &FrameBuffer<ColorT> {
        &self.background_buffer
    }

    pub fn sprite_buffer(&self) -> &FrameBuffer<ColorT> {
        &self.sprite_buffer
    }

    pub fn pattern_source_debug_buffer(&self) -> &DebugBuffer<{PixelColumn::COLUMN_COUNT}, {PixelRow::ROW_COUNT}> {
        &self.pattern_source_debug_buffer
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
            let attributes = self.oam_registers[self.oam_register_index].attributes();
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
            // VBlank scanlines. TODO: use correct address based upon pattern index.
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

enum RenderingRegisterField {
    PatternIndex,
    PaletteIndex,
    BackgroundPatternLow,
    BackgroundPatternHighAndNextTile,
    SpritePatternLow,
    SpritePatternHighAndNextSprite,
}
