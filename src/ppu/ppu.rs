use log::{info, log_enabled};
use log::Level::Info;
use splitbits::splitbits;
use ux::u5;

use crate::mapper::mapper::Mapper;
use crate::bus::Bus;
use crate::master_clock::MasterClock;
use crate::memory::cpu::cpu_address::FriendlyCpuAddress;
use crate::memory::cpu::cpu_pinout::ReadWrite;
use crate::memory::ppu::chr_memory::{PeekSource, PpuPeek};
use crate::memory::ppu::ppu_address::PpuAddress;
use crate::memory::ppu::ppu_pinout::PpuPinout;
use crate::memory::regions::secondary_oam::{SecondaryOam, SpriteField};
use crate::memory::signal_level::SignalLevel;
use crate::ppu::cycle_action::cycle_action::CycleAction;
use crate::ppu::cycle_action::frame_actions::{FrameActions, NTSC_FRAME_ACTIONS};
use crate::ppu::name_table::name_table_quadrant::NameTableQuadrant;
use crate::ppu::palette::color::Color;
use crate::ppu::palette::rgb::Rgb;
use crate::ppu::palette::rgbt::Rgbt;
use crate::ppu::palette::color_t::ColorT;
use crate::ppu::pattern_table_side::PatternTableSide;
use crate::ppu::pixel_index::{PixelColumn, PixelIndex, PixelRow};
use crate::ppu::register::ppu_registers::{AddressIncrement, ExtPinRole, PpuRegisters, Toggle};
use crate::ppu::register::registers::attribute_register::AttributeRegister;
use crate::ppu::register::registers::pattern_register::PatternRegister;
use crate::ppu::render::frame::{DebugBuffer, FrameBuffer, Frame};
use crate::ppu::sprite::oam_address::OamAddress;
use crate::ppu::sprite::sprite_attributes::{Priority, SpriteAttributes};
use crate::ppu::sprite::oam_registers::OamRegisters;
use crate::ppu::sprite::sprite_y::SpriteY;
use crate::ppu::sprite::sprite_height::SpriteHeight;
use crate::ppu::tile_number::TileNumber;
use crate::ppu::vblank_interrupt_handler::{VBlankInterruptHandler, VBlankInterruptInputSignals};
use crate::signals::{ScanlineType, SpriteRenderingDotType};

use super::palette::bank_color_assigner::BankColorAssigner;

pub struct Ppu {
    oam_registers: OamRegisters,

    next_tile_number: TileNumber,
    pattern_register: PatternRegister,
    attribute_register: AttributeRegister,
    next_rendering_field_to_set: Option<(RenderingRegisterField, u8)>,
    next_register_value: u8,
    next_register_source: PeekSource,
    pending_register_shift: bool,

    next_sprite_tile_number: TileNumber,
    current_sprite_y: SpriteY,
    sprite_visible: bool,
    sprite_0_present: bool,
    sprite_field: SpriteField,
    sprite_overflowed_this_scanline: bool,

    oam_row_corruption_pending: bool,
    oam_row_corruption_target: Option<u5>,

    next_fetch_cadence_addr: PpuAddress,

    start_vblank: SignalLevel, // Active low
    end_vblank: SignalLevel, // Active high
    pub vblank_interrupt_handler: VBlankInterruptHandler,
    //pub dummy_cpu_pinout: CpuPinout,

    frame_actions: FrameActions,

    // Used only for debug screens
    background_buffer: FrameBuffer<ColorT>,
    sprite_buffer: FrameBuffer<ColorT>,
    pattern_source_debug_buffer: DebugBuffer<{PixelColumn::COLUMN_COUNT}, {PixelRow::ROW_COUNT}>,
    bank_color_assigner: BankColorAssigner,
}

impl Ppu {
    pub fn new(bank_color_assigner: BankColorAssigner) -> Self {
        Self {
            oam_registers: OamRegisters::new(),

            next_tile_number: TileNumber::new(0),
            pattern_register: PatternRegister::new(),
            attribute_register: AttributeRegister::new(),
            next_rendering_field_to_set: None,
            next_register_value: 0,
            next_register_source: PeekSource::Void,
            pending_register_shift: false,

            next_sprite_tile_number: TileNumber::new(0),
            current_sprite_y: SpriteY::new(0),
            sprite_visible: false,
            sprite_0_present: false,
            sprite_field: SpriteField::Y,
            sprite_overflowed_this_scanline: false,

            oam_row_corruption_pending: false,
            oam_row_corruption_target: None,

            next_fetch_cadence_addr: PpuAddress::ZERO,

            start_vblank: SignalLevel::High,
            end_vblank: SignalLevel::Low,
            vblank_interrupt_handler: VBlankInterruptHandler::new(),
            //dummy_cpu_pinout: CpuPinout::new(),

            frame_actions: NTSC_FRAME_ACTIONS.clone(),

            background_buffer: FrameBuffer::default(),
            sprite_buffer: FrameBuffer::default(),
            pattern_source_debug_buffer: DebugBuffer::new(Rgb::BLACK),
            bank_color_assigner,
        }
    }

    pub fn ppusel(bus: &mut Bus) {
        use ReadWrite::*;
        use FriendlyCpuAddress::*;
        match (bus.cpu_pinout.read_write, bus.cpu_address_bus().to_friendly()) {
            (Read, PpuStatus) => { /* TODO */ }
            (Read, OamData) => { /* Nothing to do until the end of the read. */ }
            (Read, PpuData) => { /* TODO */ }
            (Read, PpuControl | PpuMask | OamAddress | PpuScroll) => { /* Write-only */ }
            (Read, _) => { /* Do nothing. */ }

            (Write, PpuControl) => { /* TODO */ }
            (Write, PpuMask) => { /* TODO */ }
            (Write, OamAddress) => { /* TODO */ }
            (Write, OamData) => { /* TODO */ }
            (Write, PpuScroll) => { /* TODO */ }
            (Write, PpuData) => { /* TODO */ }
            (Write, PpuStatus) => { /* Read-only */ }
            (Write, _) => { /* Do nothing. */ }
        }
    }

    pub fn cpu_data_bus_ready(bus: &mut Bus) {
        let friendly_cpu_addr = bus.cpu_address_bus().to_friendly();
        let read_write = bus.cpu_pinout.read_write;

        let ppu_regs = &mut bus.ppu_regs;

        use ReadWrite::*;
        use FriendlyCpuAddress::*;
        match (read_write, friendly_cpu_addr) {
            (Read, PpuStatus) => { /* TODO */ }
            (Read, OamData) => { /* Nothing to do until the end of the read. */ }
            (Read, PpuData) => { /* TODO */ }
            (Read, PpuControl | PpuMask | OamAddress | PpuScroll) => { /* Write-only */ }
            (Read, _) => { /* Do nothing. */ }

            (Write, PpuControl) => {
                let fields = splitbits!(ppu_regs.ppu_io_bus.value(), "nehb siqq");
                if !ppu_regs.nmi_enabled() && fields.n {
                    log::info!("{} NMI enabled", bus.master_clock.ppu_clock());
                }

                ppu_regs.set_nmi_enabled(&mut bus.cpu_pinout, fields.n);
                let input_signals = bus.ppu.vblank_interrupt_input_signals_from_components(&mut bus.ppu_pinout, ppu_regs);
                bus.ppu.vblank_interrupt_handler.maybe_reset(&mut bus.cpu_pinout, input_signals);

                ppu_regs.ext_pin_role              = [ExtPinRole::Read       , ExtPinRole::Write      ][fields.e as usize];
                ppu_regs.sprite_height             = [SpriteHeight::Normal   , SpriteHeight::Tall     ][fields.h as usize];
                ppu_regs.background_table_side     = [PatternTableSide::Left , PatternTableSide::Right][fields.b as usize];
                ppu_regs.sprite_table_side         = [PatternTableSide::Left , PatternTableSide::Right][fields.s as usize];
                ppu_regs.current_address_increment = [AddressIncrement::Right, AddressIncrement::Down ][fields.i as usize];
                ppu_regs.base_name_table_quadrant =  NameTableQuadrant::ALL[fields.q as usize];

                ppu_regs.next_address.set_name_table_quadrant(ppu_regs.base_name_table_quadrant);
            }
            (Write, PpuMask) => { /* TODO */ }
            (Write, OamAddress) => { /* TODO */ }
            (Write, OamData) => { /* TODO */ }
            (Write, PpuScroll) => { /* TODO */ }
            (Write, PpuData) => { /* TODO */ }
            (Write, PpuStatus) => { /* Read-only */ }
            (Write, _) => { /* Do nothing. */ }
        }
    }

    pub fn step_pclk_high(bus: &mut Bus, mapper: &mut dyn Mapper, frame: &mut Frame) {
        bus.signals.start_pclk_high(&mut bus.ppu_pinout);

        let input_signals = Ppu::vblank_interrupt_input_signals(bus);
        bus.ppu.vblank_interrupt_handler.step_pclk_high(&mut bus.cpu_pinout, input_signals);

        let tick_result = bus.ppu_regs.tick(&mut bus.signals, bus.master_clock.ppu_clock());
        if tick_result.rendering_toggled == Some(Toggle::Disable) {
            // "... when rendering is disabled, the value on the PPU address bus is the current value of the v register."
            bus.set_ppu_address_bus(mapper, bus.ppu_regs.current_address);
        }

        if log_enabled!(target: "ppusteps", Info) {
            info!(" {}\t{}", bus.ppu_clock(), bus.ppu.frame_actions.format_current_cycle_actions(bus.ppu_clock()));
        }

        if (bus.ppu_regs.background_enabled() || bus.ppu_regs.sprites_enabled())
            && bus.signals.scanline_type() != ScanlineType::Vblank
            && let Some(row) = bus.ppu.oam_row_corruption_target.take() {

            let value = bus.secondary_oam.peek_head();
            bus.secondary_oam.write_at(row, value);
            info!("Corrupting sprites {:X} and {:X} in OAM", 2 * u8::from(row), 2 * u8::from(row) + 1);
            for i in 0..8 {
                let value = bus.oam.peek(OamAddress::from_u8(i));
                bus.oam.write(OamAddress::from_u8(8 * u8::from(row) + i), value);
            }
        }

        if bus.ppu.oam_row_corruption_pending {
            bus.ppu.oam_row_corruption_pending = false;
            bus.ppu.oam_row_corruption_target = Some(bus.secondary_oam.oam_row_corruption_target(&bus.signals));
        }

        if bus.ppu_regs.take_rendering_disabled_outside_of_vblank() {
            bus.ppu.oam_row_corruption_pending = true;
        }

        bus.ppu_pinout.fetch_cadence_ale = false;
        bus.ppu_pinout.fetch_cadence_read = false;

        // TODO: Figure out how to eliminate duplication and the index.
        let len = bus.ppu.frame_actions.current_cycle_actions(bus.ppu_clock()).len();
        for i in 0..len {
            let cycle_action = bus.ppu.frame_actions.current_cycle_actions(bus.master_clock.ppu_clock())[i];
            Ppu::execute_cycle_action(bus, frame, cycle_action);
        }

        if bus.ppu_pinout.fetch_cadence_read {
            let PpuPeek { value, source } = bus.ppu_read(mapper);
            bus.ppu.next_register_value = value.resolve_with_zeros();
            bus.ppu.next_register_source = source;
        }

        if bus.ppu_pinout.fetch_cadence_ale {
            bus.set_ppu_address_bus(mapper, bus.ppu.next_fetch_cadence_addr)
        } else if bus.ppu_pinout.ppu_data_read_ale || bus.ppu_pinout.ppu_data_write_ale {
            bus.set_ppu_address_bus(mapper, bus.ppu_regs.current_address)
        }

        if bus.ppu_regs.read_2002 {
            //log::info!("\t{} Suppressed VBlank active.", bus.ppu_clock());
            bus.ppu_regs.read_2002 = false;
            let input_signals = Ppu::vblank_interrupt_input_signals(bus);
            bus.ppu.vblank_interrupt_handler.maybe_reset(&mut bus.cpu_pinout, input_signals);
            bus.ppu_regs.set_vblank_active(&mut bus.cpu_pinout, false);
        }

        mapper.on_end_of_ppu_cycle();
    }

    pub fn step_pclk_low(Bus { ppu, ppu_regs, ppu_pinout, cpu_pinout, master_clock, signals, secondary_oam, oam_buffer, secondary_oam_buffer, .. }: &mut Bus) {
        signals.start_pclk_low(ppu_pinout);

        if ppu.start_vblank.is_low() || ppu.end_vblank.is_high() {
            log::info!("Clearing start_ or end_vblank.");
            ppu.start_vblank = SignalLevel::High;
            ppu.end_vblank = SignalLevel::Low;
            let input_signals = ppu.vblank_interrupt_input_signals_from_components(ppu_pinout, ppu_regs);
            ppu.vblank_interrupt_handler.maybe_reset(cpu_pinout, input_signals);
        }

        if master_clock.ppu_clock().cycle() == 1 {
            match master_clock.ppu_clock().scanline() {
                241 => {
                    log::info!("Starting vblank");
                    ppu.start_vblank = SignalLevel::Low;
                }
                261 => {
                    log::info!("Ending vblank");
                    ppu.end_vblank = SignalLevel::High;
                    let input_signals = ppu.vblank_interrupt_input_signals_from_components(ppu_pinout, ppu_regs);
                    ppu.vblank_interrupt_handler.maybe_reset(cpu_pinout, input_signals);
                }
                _ => { /* Do nothing. */ }
            }
        }

        ppu.vblank_interrupt_handler.step_pclk_low();

        if ppu_regs.background_enabled() || ppu_regs.sprites_enabled() {
            if signals.scanline_type() == ScanlineType::Visible {
                *oam_buffer = match master_clock.ppu_clock().cycle() {
                    0 | 320.. => secondary_oam.peek(),
                    1..=64 => 0xFF,
                    65..=319 => *secondary_oam_buffer,
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

        let value = ppu.next_register_value;
        let source = ppu.next_register_source;
        match next_rendering_field_to_set {
            RenderingRegisterField::PatternIndex => {
                ppu.next_tile_number = TileNumber::new(value);
            }
            RenderingRegisterField::PaletteIndex => {
                let index = ppu_regs.current_address.to_palette_table_index(value);
                ppu.attribute_register.set_pending_palette_table_index(index);
            }
            RenderingRegisterField::BackgroundPatternLow => {
                ppu.pattern_register.set_pending_low_byte(value, source);
            }
            RenderingRegisterField::BackgroundPatternHighAndNextTile => {
                ppu.pattern_register.set_pending_high_byte(value, source);
                ppu.attribute_register.prepare_next_palette_table_index();
                ppu.pattern_register.load_next_palette_indexes();
                ppu_regs.current_address.increment_coarse_x_scroll();
            }
            RenderingRegisterField::SpritePatternLow => {
                // TODO: Determine if this check should happen at the same time as rendering_enabled()
                if ppu.sprite_visible {
                    ppu.oam_registers[secondary_oam.oam_register_index()].set_pattern_low(value, source);
                }
            }
            RenderingRegisterField::SpritePatternHighAndNextSprite => {
                // TODO: Determine if this check should happen at the same time as rendering_enabled()
                if ppu.sprite_visible {
                    ppu.oam_registers[secondary_oam.oam_register_index()].set_pattern_high(value, source);
                }
            }
        }
    }

    fn execute_cycle_action(
        Bus { ppu, ppu_regs, ppu_pinout, cpu_pinout, master_clock, signals, palette_ram, ciram, oam,
            secondary_oam, secondary_oam_buffer, composite_decoders, .. }: &mut Bus,
        frame: &mut Frame,
        cycle_action: CycleAction
    ) {
        let clock = master_clock.ppu_clock();

        use CycleAction::*;
        match cycle_action {
            SetPatternIndexAddress => {
                if signals.rendering_enabled {
                    ppu_pinout.fetch_cadence_ale = true;
                }

                ppu.next_rendering_field_to_set = Some((RenderingRegisterField::PatternIndex, 2));
                ppu.next_fetch_cadence_addr = ppu_regs.current_address.to_name_table_address();
            }
            SetPaletteIndexAddress => {
                if signals.rendering_enabled {
                    ppu_pinout.fetch_cadence_ale = true;
                    ppu.next_rendering_field_to_set = Some((RenderingRegisterField::PaletteIndex, 2));
                }

                ppu.next_fetch_cadence_addr = ppu_regs.current_address.to_attribute_table_address();
            }
            SetPatternLowAddress => {
                if signals.rendering_enabled {
                    ppu_pinout.fetch_cadence_ale = true;
                    ppu.next_rendering_field_to_set = Some((RenderingRegisterField::BackgroundPatternLow, 2));
                }

                ppu.next_fetch_cadence_addr = PpuAddress::in_pattern_table(
                    ppu_regs.background_table_side(),
                    ppu.next_tile_number,
                    ppu_regs.current_address.fine_y_scroll(),
                    false,
                );
            }
            SetPatternHighAddress => {
                if signals.rendering_enabled {
                    ppu_pinout.fetch_cadence_ale = true;
                    ppu.next_rendering_field_to_set = Some((RenderingRegisterField::BackgroundPatternHighAndNextTile, 2));
                }

                ppu.next_fetch_cadence_addr = PpuAddress::in_pattern_table(
                    ppu_regs.background_table_side(),
                    ppu.next_tile_number,
                    ppu_regs.current_address.fine_y_scroll(),
                    true,
                );
            }

            Read => {
                if signals.rendering_enabled {
                    ppu_pinout.fetch_cadence_read = true;
                }
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
                        let rgb = ppu.bank_color_assigner.rgb_for_source(ppu.pattern_register.current_source());
                        Rgbt::Opaque(rgb)
                    });
                }

                // This is not delayed, unlike signals.rendering_enabled
                let rendering_enabled = ppu_regs.background_enabled() || ppu_regs.sprites_enabled();

                let should_shift_sprites = rendering_enabled
                    && matches!(signals.scanline_type(), ScanlineType::Visible | ScanlineType::Prerender)
                    && matches!(signals.sprite_rendering_dot_type(), SpriteRenderingDotType::OamClearing | SpriteRenderingDotType::Evaluation);
                let (mut sprite_color, sprite_priority, is_sprite_0, ppu_peek_source) = ppu.oam_registers.step(palette_ram, should_shift_sprites);
                let mut pixel_color;
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

                    pixel_color = match (background_color, sprite_color, sprite_priority) {
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
                        let rgb = ppu.bank_color_assigner.rgb_for_source(ppu_peek_source);
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
                    pixel_color = ppu_regs.current_address.to_palette_ram_index()
                        .map(|palette_ram_index| palette_ram.peek_color(ppu_regs.mask(), palette_ram_index))
                        .unwrap_or(palette_ram.backdrop_color());
                }

                if !composite_decoders.show_overscan && pixel_index.is_in_overscan_region() {
                    pixel_color = Color::BLACK;
                }

                composite_decoders.get_mut().set_color(frame, &master_clock, pixel_color, ppumask.emphasis());
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

            // TODO: There's no test ROM that verifies that this happens even if rendering is disabled.
            ResetSecondaryOamIndex => {
                secondary_oam.reset_index();
            }
            StartSpriteEvaluation => {
                info!(target: "ppustage", "\t\tSPRITE EVALUATION");
                ppu.sprite_0_present = false;
                ppu_regs.oam_addr.clear_all_sprites_evaluated();
                secondary_oam.reset_index();
                ppu.sprite_field = SpriteField::Y;
            }
            StartLoadingOamRegisters => {
                info!(target: "ppustage", "\t\tLoading OAM registers.");
                if ppu_regs.background_enabled() || ppu_regs.sprites_enabled() {
                    ppu.oam_registers.set_sprite_0_presence(ppu.sprite_0_present);
                }

                // TODO: Determine if this needs to occur on cycle 256 instead.
                secondary_oam.reset_index();
                // Sprite eval is done, so might as well clear this now rather than later.
                ppu.sprite_overflowed_this_scanline = false;
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
                if signals.rendering_enabled {
                    match signals.scanline_type() {
                        ScanlineType::Visible => secondary_oam.write(*secondary_oam_buffer),
                        ScanlineType::Vblank => unreachable!(),
                        ScanlineType::Prerender => {}
                    }

                    secondary_oam.advance();
                }
            }
            ReadSpriteByte => {
                if (ppu_regs.background_enabled() || ppu_regs.sprites_enabled()) && signals.scanline_type() != ScanlineType::Prerender {
                    *secondary_oam_buffer = oam.read(ppu_regs.oam_addr);
                }
            }
            // TODO: Can this sanely be written branchless? I suspect that's cleanest as it closest models how the hardware works.
            EvaluateSpriteByte => {
                if !ppu_regs.background_enabled() && !ppu_regs.sprites_enabled() { return; }

                let mut should_read = true;
                let mut start_next_field = false;
                let mut start_next_sprite = false;
                if ppu_regs.oam_addr.all_sprites_evaluated() {
                    // Writes are disabled once all sprites are evaluated, but subsequent sprites are still visited.
                    start_next_sprite = true;
                } else {
                    if !secondary_oam.is_full() && signals.scanline_type() != ScanlineType::Prerender {
                        // Copy the sprite byte into secondary OAM (it will be overridden if it's an out-of-range Y coordinate).
                        secondary_oam.write(*secondary_oam_buffer);
                        should_read = false;
                    }

                    if ppu.sprite_field != SpriteField::Y {
                        // This sprite is already known to be in range, so keep copying fields over.
                        start_next_field = true;
                        secondary_oam.advance();
                        ppu.sprite_field.step();
                    } else if ppu.sprite_overflowed_this_scanline {
                        start_next_sprite = true;
                    } else if let Some(pixel_row) = clock.scanline_pixel_row() // Current scanline is visible
                        && let Some(top_sprite_row) = PixelRow::try_from_u8(*secondary_oam_buffer) // Top row of sprite in bounds
                        && let Some(offset) = pixel_row.difference(top_sprite_row) // Current scanline is lower than top-of-sprite
                        && offset < ppu_regs.sprite_height().to_dimension() { // Current scanline is inside of sprite

                        if clock.scanline() == 261 {
                            secondary_oam.log_all();
                        }

                        // This sprite's Y coordinate is in range, so move to its next field.
                        start_next_field = true;
                        ppu.sprite_0_present |= clock.cycle() == 66;

                        ppu.sprite_field.step();

                        ppu.sprite_overflowed_this_scanline |= secondary_oam.is_full();
                        ppu_regs.sprite_overflow_pending |= secondary_oam.is_full();
                        if !ppu_regs.sprite_overflow_pending {
                            secondary_oam.advance();
                        }
                    } else {
                        // This sprite is not in range, move to the next one.
                        start_next_sprite = true;
                        // If the sprite isn't in range and secondary OAM is full,
                        // then corrupt the OAMADDR by also incrementing field.
                        start_next_field = secondary_oam.is_full();
                    }
                }

                if should_read {
                    *secondary_oam_buffer = secondary_oam.peek();
                }

                if signals.scanline_type() != ScanlineType::Prerender {
                    let field_overflowed = if start_next_field {
                        ppu_regs.oam_addr.next_field()
                    } else {
                        false
                    };

                    if ppu.sprite_field == SpriteField::Y && ppu.sprite_overflowed_this_scanline {
                        ppu_regs.oam_addr.reset_field();
                    }

                    // If the field overflowed, then we're already on the next sprite.
                    if start_next_sprite && !field_overflowed {
                        ppu_regs.oam_addr.next_sprite();
                    }
                }
            }
            AdvanceSecondaryOam => {
                secondary_oam.advance();
            }
            ReadSpriteY => {
                if signals.rendering_enabled {
                    *secondary_oam_buffer = secondary_oam.peek();
                    ppu.current_sprite_y = SpriteY::new(*secondary_oam_buffer);
                }
            }
            ReadSpritePatternIndex => {
                if signals.rendering_enabled {
                    *secondary_oam_buffer = secondary_oam.peek();
                    ppu.next_sprite_tile_number = TileNumber::new(*secondary_oam_buffer);
                }
            }
            ReadSpriteAttributes => {
                if signals.rendering_enabled {
                    *secondary_oam_buffer = secondary_oam.peek();
                    let attributes = SpriteAttributes::from_u8(*secondary_oam_buffer);
                    ppu.oam_registers[secondary_oam.oam_register_index()].set_attributes(attributes);
                }
            }
            ReadSpriteX => {
                if signals.rendering_enabled {
                    *secondary_oam_buffer = secondary_oam.peek();
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
                    ppu_pinout.fetch_cadence_ale = true;
                    ppu.next_rendering_field_to_set = Some((RenderingRegisterField::SpritePatternLow, 2));
                }

                let select_high = false;
                let addr;
                (addr, ppu.sprite_visible) = ppu.current_sprite_pattern_address(master_clock, ppu_regs, secondary_oam, select_high);
                ppu.next_fetch_cadence_addr = addr;
            }
            SetSpritePatternHighAddress => {
                if signals.rendering_enabled {
                    ppu_pinout.fetch_cadence_ale = true;
                    ppu.next_rendering_field_to_set = Some((RenderingRegisterField::SpritePatternHighAndNextSprite, 2));
                }

                let select_high = true;
                let addr;
                (addr, ppu.sprite_visible) = ppu.current_sprite_pattern_address(master_clock, ppu_regs, secondary_oam, select_high);
                ppu.next_fetch_cadence_addr = addr;
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
                ppu_regs.set_vblank_active(cpu_pinout, true);
                // start_vblank is set during PCLK low. This is PCLK high.

                // "During VBlank ... the value on the PPU address bus is the current value of the v register."
                ppu.next_fetch_cadence_addr = ppu_regs.current_address;
            }
            SetInitialScrollOffsets => {
                if ppu_regs.background_enabled() {
                    ppu_regs.current_address = ppu_regs.next_address;
                }
            }
            SetInitialYScroll => {
                if ppu_regs.background_enabled() {
                    let next_address = ppu_regs.next_address;
                    ppu_regs.current_address.copy_y_scroll(next_address);
                }
            }

            ClearFlags => {
                ppu_regs.set_vblank_active(cpu_pinout, false);
                ppu_regs.sprite0_hit = false;
                ppu_regs.sprite_overflow = false;
                // end_vblank is set during PCLK low. This is PCLK high.

                // At startup, CIRAM writes are disabled until the PPU has been running for a while.
                // TODO: Determine the correct place to call this.
                ciram.enable_writes();
            }
        }
    }

    pub fn vblank_interrupt_input_signals(bus: &Bus) -> VBlankInterruptInputSignals {
        bus.ppu.vblank_interrupt_input_signals_from_components(&bus.ppu_pinout, &bus.ppu_regs)
    }

    pub fn vblank_interrupt_input_signals_from_components(&self, ppu_pinout: &PpuPinout, ppu_regs: &PpuRegisters) -> VBlankInterruptInputSignals {
        VBlankInterruptInputSignals {
            vset: self.start_vblank,
            vbl: if ppu_regs.nmi_enabled() { SignalLevel::High } else { SignalLevel::Low },
            vclr: self.end_vblank,
            r2: if ppu_regs.read_2002 { SignalLevel::Low } else { SignalLevel::High },
            dbe: ppu_pinout.data_bus_enabled,
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