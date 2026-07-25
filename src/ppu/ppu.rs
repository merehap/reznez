use log::{info, log_enabled};
use log::Level::Info;
use ux::u5;

use crate::mapper::mapper::Mapper;
use crate::bus::Bus;
use crate::master_clock::MasterClock;
use crate::memory::ppu::chr_memory::PpuPeek;
use crate::memory::ppu::ppu_address::{PaletteRamIndex, PpuAddress};
use crate::memory::regions::secondary_oam::{SecondaryOam, SpriteField};
use crate::memory::signal_level::SignalLevel;
use crate::ppu::cycle_action::cycle_action::CycleAction;
use crate::ppu::cycle_action::frame_actions::{FrameActions, NTSC_FRAME_ACTIONS};
use crate::ppu::palette::color::Color;
use crate::ppu::palette::rgb::Rgb;
use crate::ppu::palette::rgbt::Rgbt;
use crate::ppu::palette::color_t::ColorT;
use crate::ppu::pattern_table_side::PatternTableSide;
use crate::ppu::pixel_index::{PixelColumn, PixelIndex, PixelRow};
use crate::ppu::register::ppu_registers::{PpuRegisters, Toggle};
use crate::ppu::register::registers::attribute_register::AttributeRegister;
use crate::ppu::register::registers::pattern_register::PatternRegister;
use crate::ppu::render::frame::{DebugBuffer, FrameBuffer, Frame};
use crate::ppu::sprite::sprite_attributes::{Priority, SpriteAttributes};
use crate::ppu::sprite::oam_registers::OamRegisters;
use crate::ppu::sprite::sprite_y::SpriteY;
use crate::ppu::sprite::sprite_height::SpriteHeight;
use crate::ppu::tile_number::TileNumber;
use crate::signals::ScanlineType;

use super::palette::bank_color_assigner::BankColorAssigner;

pub struct Ppu {
    oam_registers: OamRegisters,

    next_tile_number: TileNumber,
    pattern_register: PatternRegister,
    attribute_register: AttributeRegister,
    next_rendering_field_to_set: Option<(RenderingRegisterField, u8)>,
    next_register_value: PpuPeek,
    pending_register_shift: bool,

    next_sprite_tile_number: TileNumber,
    current_sprite_y: SpriteY,
    sprite_visible: bool,

    // Sprite evaluation
    all_sprites_evaluated: bool,
    sprite_0_present: bool,

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

            next_tile_number: TileNumber::new(0),
            pattern_register: PatternRegister::new(),
            attribute_register: AttributeRegister::new(),
            next_rendering_field_to_set: None,
            next_register_value: PpuPeek::VOID,
            pending_register_shift: false,

            next_sprite_tile_number: TileNumber::new(0),
            current_sprite_y: SpriteY::new(0),
            sprite_visible: false,

            all_sprites_evaluated: false,
            sprite_0_present: false,

            frame_actions: NTSC_FRAME_ACTIONS.clone(),

            background_buffer: FrameBuffer::default(),
            sprite_buffer: FrameBuffer::default(),
            pattern_source_debug_buffer: DebugBuffer::new(Rgb::BLACK),
            bank_color_assigner,
        }
    }

    pub fn step_first_half(bus: &mut Bus, mapper: &mut dyn Mapper, frame: &mut Frame) {
        let tick_result = bus.ppu_regs.tick(&mut bus.signals, bus.master_clock.ppu_clock());
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
            match Ppu::execute_cycle_action(bus, frame, cycle_action) {
                Followup::None => {}
                Followup::PpuReadNextRegisterValue => bus.ppu.next_register_value = bus.ppu_read(mapper),
                Followup::SetAddressBus(addr) => bus.set_ppu_address_bus(mapper, addr),
            }
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

    pub fn step_second_half(Bus { ppu, ppu_regs, master_clock, signals, secondary_oam, oam_buffer, secondary_oam_buffer, .. }: &mut Bus) {
        if ppu_regs.background_enabled() || ppu_regs.sprites_enabled() {
            if signals.scanline_type() == ScanlineType::Visible {
                *oam_buffer = match master_clock.ppu_clock().cycle() {
                    0 | 320.. => secondary_oam.peek(),
                    1..=65 => 0xFF,
                    66..=319 => *secondary_oam_buffer,
                };
            }

            if ppu.pending_register_shift {
                ppu.pending_register_shift = false;
                ppu.pattern_register.shift_left();
                ppu.attribute_register.push_next_palette_table_index();
            }
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
                    ppu.oam_registers[secondary_oam.oam_register_index()].set_pattern_low(ppu.next_register_value);
                }
            }
            RenderingRegisterField::SpritePatternHighAndNextSprite => {
                // TODO: Determine if this check should happen at the same time as rendering_enabled()
                if ppu.sprite_visible {
                    ppu.oam_registers[secondary_oam.oam_register_index()].set_pattern_high(ppu.next_register_value);
                }
            }
        }
    }

    fn execute_cycle_action(
        Bus { ppu, ppu_regs, master_clock, signals, palette_ram, ciram, oam, secondary_oam, secondary_oam_buffer, composite_decoders, .. }: &mut Bus,
        frame: &mut Frame,
        cycle_action: CycleAction
    ) -> Followup {
        let mut followup = Followup::None;

        let clock = master_clock.ppu_clock();

        use CycleAction::*;
        match cycle_action {
            SetPatternIndexAddress => {
                ppu.next_rendering_field_to_set = Some((RenderingRegisterField::PatternIndex, 2));
                followup = Followup::SetAddressBus(ppu_regs.current_address.to_name_table_address());
            }
            SetPaletteIndexAddress => {
                if signals.rendering_enabled {
                    ppu.next_rendering_field_to_set = Some((RenderingRegisterField::PaletteIndex, 2));
                }

                followup = Followup::SetAddressBus(ppu_regs.current_address.to_attribute_table_address());
            }
            SetPatternLowAddress => {
                if signals.rendering_enabled {
                    ppu.next_rendering_field_to_set = Some((RenderingRegisterField::BackgroundPatternLow, 2));
                }

                followup = Followup::SetAddressBus(PpuAddress::in_pattern_table(
                    ppu_regs.background_table_side(),
                    ppu.next_tile_number,
                    ppu_regs.current_address.fine_y_scroll(),
                    false,
                ));
            }
            SetPatternHighAddress => {
                if signals.rendering_enabled {
                    ppu.next_rendering_field_to_set = Some((RenderingRegisterField::BackgroundPatternHighAndNextTile, 2));
                }

                followup = Followup::SetAddressBus(PpuAddress::in_pattern_table(
                    ppu_regs.background_table_side(),
                    ppu.next_tile_number,
                    ppu_regs.current_address.fine_y_scroll(),
                    true,
                ));
            }

            Read => {
                followup = Followup::PpuReadNextRegisterValue;
            }
            PrepareForNextPixel => {
                ppu.pending_register_shift = true;
            }

            GotoNextPixelRow => {
                if signals.rendering_enabled {
                    ppu_regs.current_address.increment_fine_y_scroll();
                }
            }
            ResetTileColumn => {
                if signals.rendering_enabled {
                    ppu_regs.current_address.set_tile_column_from(ppu_regs.next_address);
                }
            }
            SetPixel => {
                let pixel_index = PixelIndex::try_from_clock(clock).unwrap();
                let ppumask = ppu_regs.mask();

                let mut background_color = ColorT::Transparent;
                let mut background_bank_pixel = None;
                if ppu_regs.background_enabled() {
                    let column_in_tile = ppu_regs.fine_x_scroll;
                    let palette_table_index = ppu.attribute_register.palette_table_index(column_in_tile);
                    let palette = palette_ram.background_palette(palette_table_index);

                    background_color = ppu.pattern_register
                        .palette_index(column_in_tile)
                        .map_or(ColorT::Transparent, |palette_index| ColorT::Opaque(palette[palette_index]));
                    background_bank_pixel = Some(if background_color.is_transparent() {
                        Rgbt::Transparent
                    } else {
                        let rgb = ppu.bank_color_assigner.rgb_for_source(ppu.pattern_register.current_peek().source());
                        Rgbt::Opaque(rgb)
                    });
                }

                // This is not delayed, unlike signals.rendering_enabled
                let rendering_enabled = ppu_regs.background_enabled() || ppu_regs.sprites_enabled();
                let (mut sprite_color, sprite_priority, is_sprite_0, ppu_peek) =
                    ppu.oam_registers.step(palette_ram, rendering_enabled);
                let mut color;
                if rendering_enabled {
                    if !ppu_regs.sprites_enabled() {
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

                    // TODO: This should probably be a natural consequence of the shifter pipeline instead.
                    if row == PixelRow::ZERO {
                        sprite_color = Transparent;
                    }

                    color = match (background_color, sprite_color, sprite_priority) {
                        (Transparent  , Transparent  , _) => palette_ram.backdrop_color(),
                        (Transparent  , Opaque(color), _) => color,
                        (Opaque(color), Transparent  , _) => color,
                        (Opaque(_)    , Opaque(color), Priority::InFront) => color,
                        (Opaque(color), Opaque(_)    , Priority::Behind ) => color,
                    };

                    // These two are just for debug screens.
                    ppu.background_buffer[pixel_index] = background_color;
                    ppu.sprite_buffer[pixel_index] = sprite_color;

                    // https://wiki.nesdev.org/w/index.php?title=PPU_OAM#Sprite_zero_hits
                    let sprite_0_hit =
                        is_sprite_0 &&
                        column < PixelColumn::MAX &&
                        row <= PixelRow::MAX &&
                        background_color.is_opaque() &&
                        sprite_color.is_opaque();
                    if sprite_0_hit {
                        ppu_regs.sprite0_hit_pending = true;
                    }

                    let sprite_bank_pixel = Some(if sprite_color.is_transparent() {
                        Rgbt::Transparent
                    } else {
                        let rgb = ppu.bank_color_assigner.rgb_for_source(ppu_peek.source());
                        Rgbt::Opaque(rgb)
                    });
                    let is_sprite_pixel = background_color.is_transparent()
                        || (sprite_color.is_opaque() && sprite_priority == Priority::InFront);
                    let bank_pixel = if is_sprite_pixel { sprite_bank_pixel } else { background_bank_pixel };
                    if let Some(bank_pixel) = bank_pixel {
                        let column = pixel_index.column.to_usize();
                        let row = pixel_index.row.to_usize();
                        ppu.pattern_source_debug_buffer.write_rgbt(column, row, bank_pixel);
                    }
                } else {
                    color = if ppu_regs.current_address.is_in_palette_table() {
                        let index = PaletteRamIndex::new(u5::new((ppu_regs.current_address.to_u16() & 0b0001_1111) as u8));
                        Color::from(palette_ram.peek(ppu_regs, index).value())
                    } else {
                        palette_ram.backdrop_color()
                    };
                }

                if !composite_decoders.show_overscan && pixel_index.is_in_overscan_region() {
                    color = Color::BLACK;
                }

                composite_decoders.get_mut().set_color(frame, &master_clock, color, ppumask.emphasis());
            }

            MaybeCorruptOamStart => {
                if signals.rendering_enabled {
                    // Unclear if these are the correct cycles to trigger on.
                    let oam_addr = ppu_regs.oam_addr;
                    oam.maybe_corrupt_starting_byte(clock, oam_addr);
                }
            }

            ResetOamAddress => {
                if ppu_regs.background_enabled() || ppu_regs.sprites_enabled() {
                    ppu_regs.oam_addr.reset();
                }
            }

            ResetSecondaryOamIndex => {
                if signals.rendering_enabled {
                    secondary_oam.reset_index();
                }
            }
            StartSpriteEvaluation => {
                info!(target: "ppustage", "\t\tSPRITE EVALUATION");
                ppu.sprite_0_present = false;
                ppu.all_sprites_evaluated = false;
                secondary_oam.reset_index();
            }
            StartLoadingOamRegisters => {
                info!(target: "ppustage", "\t\tLoading OAM registers.");
                ppu.oam_registers.set_sprite_0_presence(ppu.sprite_0_present);
                // TODO: Determine if this needs to occur on cycle 256 instead.
                secondary_oam.reset_index();
            }
            StopLoadingOamRegisters => {
                info!(target: "ppustage", "\t\tLoading OAM registers ended.");
            }
            ClearSpriteBuffer => {
                if signals.rendering_enabled {
                    *secondary_oam_buffer = match signals.scanline_type() {
                        ScanlineType::Visible   => 0xFF,
                        ScanlineType::Vblank    => unreachable!(),
                        ScanlineType::Prerender => secondary_oam.peek(),
                    };
                }
            }
            WriteSpriteByte => {
                if ppu_regs.background_enabled() || ppu_regs.sprites_enabled() {
                    match signals.scanline_type() {
                        ScanlineType::Visible => secondary_oam.write(*secondary_oam_buffer),
                        ScanlineType::Vblank => unreachable!(),
                        ScanlineType::Prerender => {}
                    }

                    secondary_oam.advance();
                }
            }
            ReadSpriteByte => {
                if signals.rendering_enabled {
                    *secondary_oam_buffer = oam.read(ppu_regs.oam_addr);
                }
            }
            EvaluateSpriteByte => {
                if !ppu_regs.background_enabled() && !ppu_regs.sprites_enabled() { return Followup::None; }

                if ppu.all_sprites_evaluated {
                    secondary_oam.read(secondary_oam_buffer);
                    secondary_oam.advance();
                    return Followup::None;
                }

                if !secondary_oam.is_full() {
                    // Copy the sprite byte into secondary OAM, but it may be overwritten
                    // if it is a Y coordinate that is not in range.
                    secondary_oam.write(*secondary_oam_buffer);
                }

                // If the Y-index is not currently being evaluated, then the current sprite was already verified to be on screen,
                // so copy another byte of its data over.
                if secondary_oam.current_field() != SpriteField::Y {
                    secondary_oam.advance();
                    ppu.all_sprites_evaluated = ppu_regs.oam_addr.next_field();
                    return Followup::None;
                }

                // Check if the sprite's Y-index is on screen and if the current row is inside of the sprite.
                if let Some(pixel_row) = clock.scanline_pixel_row()
                    && let Some(top_sprite_row) = PixelRow::try_from_u8(*secondary_oam_buffer)
                    && let Some(offset) = pixel_row.difference(top_sprite_row)
                    && offset < ppu_regs.sprite_height().to_dimension()
                {
                    if clock.cycle() == 66 {
                        ppu.sprite_0_present = true;
                    }

                    if secondary_oam.is_full() {
                        ppu_regs.sprite_overflow_pending = true;
                    }

                    secondary_oam.advance();
                    ppu.all_sprites_evaluated = ppu_regs.oam_addr.next_field();
                    return Followup::None;
                }

                // If the sprite isn't in range and OAM is full, then corrupt the OAMADDR.
                // Sprite overflow hardware bug: https://www.nesdev.org/wiki/PPU_sprite_evaluation#Details
                if secondary_oam.is_full() {
                    ppu_regs.oam_addr.corrupt_sprite_y_index();
                }

                // If the sprite isn't in range, move to the next sprite.
                ppu.all_sprites_evaluated = ppu_regs.oam_addr.next_sprite();
            }
            AdvanceSecondaryOam => {
                secondary_oam.advance();
            }
            ReadSpriteY => {
                if signals.rendering_enabled {
                    secondary_oam.read(secondary_oam_buffer);
                    ppu.current_sprite_y = SpriteY::new(*secondary_oam_buffer);
                }
            }
            ReadSpritePatternIndex => {
                if signals.rendering_enabled {
                    secondary_oam.read(secondary_oam_buffer);
                    ppu.next_sprite_tile_number = TileNumber::new(*secondary_oam_buffer);
                }
            }
            ReadSpriteAttributes => {
                if signals.rendering_enabled {
                    secondary_oam.read(secondary_oam_buffer);
                    let attributes = SpriteAttributes::from_u8(*secondary_oam_buffer);
                    ppu.oam_registers[secondary_oam.oam_register_index()].set_attributes(attributes);
                }
            }
            ReadSpriteX => {
                if signals.rendering_enabled {
                    secondary_oam.read(secondary_oam_buffer);
                    ppu.oam_registers[secondary_oam.oam_register_index()].set_x_counter(*secondary_oam_buffer);
                }
            }
            DummyReadSpriteX => {
                // TODO
            }
            MaybeClearSpriteX => {
                if !signals.rendering_enabled {
                    // This is a quirk of the rendering pipeline. There may be a better way to represent this.
                    for i in 0..8 {
                        ppu.oam_registers[i].set_x_counter(0);
                    }
                }
            }

            SetSpritePatternLowAddress => {
                if signals.rendering_enabled {
                    ppu.next_rendering_field_to_set = Some((RenderingRegisterField::SpritePatternLow, 2));
                }

                let select_high = false;
                let addr;
                (addr, ppu.sprite_visible) = ppu.current_sprite_pattern_address(master_clock, ppu_regs, secondary_oam, select_high);
                followup = Followup::SetAddressBus(addr)
            }
            SetSpritePatternHighAddress => {
                if signals.rendering_enabled {
                    ppu.next_rendering_field_to_set = Some((RenderingRegisterField::SpritePatternHighAndNextSprite, 2));
                }

                let select_high = true;
                let addr;
                (addr, ppu.sprite_visible) = ppu.current_sprite_pattern_address(master_clock, ppu_regs, secondary_oam, select_high);
                followup = Followup::SetAddressBus(addr);
            }

            // TODO: Remove this section in favor of using EdgeDetectors.
            StartVisibleScanlines => {
                info!(target: "ppustage", "{}\tVISIBLE SCANLINES", master_clock.ppu_clock());
            }
            StartPostRenderScanline => {
                info!(target: "ppustage", "{}\tPOST-RENDER SCANLINE", master_clock.ppu_clock());
                // Move OAM towards its decayed state once per frame. Placing this here is arbitrary.
                oam.maybe_decay();
            }
            StartVblankScanlines => {
                info!(target: "ppustage", "{}\tVBLANK SCANLINES", clock);
            }
            StartPreRenderScanline => {
                info!(target: "ppustage", "{}\tPRE-RENDER SCANLINE", clock);
            }
            StartReadingBackgroundTiles => {
                info!(target: "ppustage", "{}\t\tREADING BACKGROUND TILES", clock);
            }
            StopReadingBackgroundTiles => {
                info!(target: "ppustage", "{}\t\tENDED READING BACKGROUND TILES", clock);
            }

            StartVblank => {
                ppu_regs.vblank_active = true;
                // "During VBlank ... the value on the PPU address bus is the current value of the v register."
                followup = Followup::SetAddressBus(ppu_regs.current_address);
            }
            SetInitialScrollOffsets => {
                if !ppu_regs.background_enabled() { return Followup::None; }
                ppu_regs.current_address = ppu_regs.next_address;
            }
            SetInitialYScroll => {
                if !ppu_regs.background_enabled() { return Followup::None; }
                let next_address = ppu_regs.next_address;
                ppu_regs.current_address.copy_y_scroll(next_address);
            }

            ClearFlags => {
                ppu_regs.vblank_active = false;
                ppu_regs.sprite0_hit = false;
                ppu_regs.sprite_overflow = false;

                // At startup, CIRAM writes are disabled until the PPU has been running for a while.
                // TODO: Determine the correct place to call this.
                ciram.enable_writes();
            }
        }

        followup
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

    fn current_sprite_pattern_address(&self, clock: &MasterClock, ppu_regs: &PpuRegisters, secondary_oam: &SecondaryOam, select_high: bool) -> (PpuAddress, bool) {
        let sprite_table_side = ppu_regs.sprite_table_side();
        let sprite_height = ppu_regs.sprite_height();
        let sprite_table_side = match sprite_height {
            SpriteHeight::Normal => sprite_table_side,
            SpriteHeight::Tall => self.next_sprite_tile_number.tall_sprite_pattern_table_side(),
        };

        let address;
        let visible;
        if let Some(pixel_row) = clock.ppu_clock().scanline_pixel_row() {
            let attributes = self.oam_registers[secondary_oam.oam_register_index()].attributes();
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


enum Followup {
    None,
    PpuReadNextRegisterValue,
    SetAddressBus(PpuAddress),
}