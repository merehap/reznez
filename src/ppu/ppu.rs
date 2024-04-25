use log::info;

use crate::memory::memory::PpuMemory;
use crate::memory::ppu::ppu_address::PpuAddress;
use crate::ppu::clock::Clock;
use crate::ppu::cycle_action::cycle_action::CycleAction;
use crate::ppu::cycle_action::frame_actions::{FrameActions, NTSC_FRAME_ACTIONS};
use crate::ppu::palette::palette_table_index::PaletteTableIndex;
use crate::ppu::palette::rgbt::Rgbt;
use crate::ppu::pattern_table::PatternIndex;
use crate::ppu::pixel_index::{PixelIndex, PixelRow};
use crate::ppu::register::registers::attribute_register::AttributeRegister;
use crate::ppu::register::registers::pattern_register::PatternRegister;
use crate::ppu::render::frame::Frame;
use crate::ppu::sprite::sprite_attributes::SpriteAttributes;
use crate::ppu::sprite::secondary_oam::SecondaryOam;
use crate::ppu::sprite::oam_registers::OamRegisters;
use crate::ppu::sprite::sprite_y::SpriteY;
use crate::ppu::sprite::sprite_height::SpriteHeight;

pub struct Ppu {
    oam_data_read: u8,
    secondary_oam: SecondaryOam,
    oam_registers: OamRegisters,
    oam_register_index: usize,
    clear_oam: bool,
    all_sprites_evaluated: bool,

    clock: Clock,

    next_pattern_index: PatternIndex,
    pattern_register: PatternRegister,
    attribute_register: AttributeRegister,

    next_sprite_pattern_index: PatternIndex,
    current_sprite_y: SpriteY,
    sprite_0_present: bool,

    frame_actions: FrameActions,
}

impl Ppu {
    pub fn new(clock: Clock) -> Ppu {
        Ppu {
            oam_data_read: 0,
            secondary_oam: SecondaryOam::new(),
            oam_registers: OamRegisters::new(),
            oam_register_index: 0,
            clear_oam: false,
            all_sprites_evaluated: false,

            clock,

            next_pattern_index: PatternIndex::new(0),
            pattern_register: PatternRegister::new(),
            attribute_register: AttributeRegister::new(),

            next_sprite_pattern_index: PatternIndex::new(0),
            current_sprite_y: SpriteY::new(0),
            sprite_0_present: false,

            frame_actions: NTSC_FRAME_ACTIONS.clone(),
        }
    }

    pub fn clock(&self) -> &Clock {
        &self.clock
    }

    pub fn step(&mut self, mem: &mut PpuMemory, frame: &mut Frame) -> StepResult {
        let is_last_cycle_of_frame = self.clock.tick(mem.regs().rendering_enabled());

        mem.regs_mut().maybe_apply_status_read(&self.clock);
        mem.regs_mut().maybe_toggle_rendering_enabled();
        mem.regs_mut().maybe_decay_ppu_io_bus(&self.clock);

        // TODO: Figure out how to eliminate duplication and the index.
        let len = self.frame_actions.current_cycle_actions(&self.clock).len();
        for i in 0..len {
            let cycle_action = self.frame_actions.current_cycle_actions(&self.clock)[i];
            info!(target: "ppusteps", " {}\t{:?}", self.clock, cycle_action);
            self.execute_cycle_action(mem, frame, cycle_action);
        }

        let should_generate_nmi = mem.regs().nmi_requested && mem.regs().can_generate_nmi();
        mem.regs_mut().nmi_requested = false;

        mem.process_end_of_ppu_cycle();
        StepResult { is_last_cycle_of_frame, should_generate_nmi }
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

        use CycleAction::*;
        match cycle_action {
            GetPatternIndex => {
                if !mem.regs().rendering_enabled() { return; }
                let address = PpuAddress::in_name_table(name_table_quadrant, tile_column, tile_row);
                self.next_pattern_index = PatternIndex::new(mem.read(address));
            }
            GetPaletteIndex => {
                if !mem.regs().rendering_enabled() { return; }
                let address = PpuAddress::in_attribute_table(name_table_quadrant, tile_column, tile_row);
                let attribute_byte = mem.read(address);
                let palette_table_index =
                    PaletteTableIndex::from_attribute_byte(attribute_byte, tile_column, tile_row);
                self.attribute_register.set_pending_palette_table_index(palette_table_index);
            }
            GetPatternLowByte => {
                if !mem.regs().rendering_enabled() { return; }
                let address = PpuAddress::in_pattern_table(
                    background_table_side, self.next_pattern_index, row_in_tile, 0x0);
                self.pattern_register.set_pending_low_byte(mem.read(address));
            }
            GetPatternHighByte => {
                if !mem.regs().rendering_enabled() { return; }
                let address = PpuAddress::in_pattern_table(
                    background_table_side, self.next_pattern_index, row_in_tile, 0x8);
                self.pattern_register.set_pending_high_byte(mem.read(address));
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
                let (pixel_column, pixel_row) = PixelIndex::try_from_clock(&self.clock).unwrap().to_column_row();
                // TODO: Verify if these need to be delayed like self.rendering_enabled.
                if background_enabled {
                    let palette_table = mem.palette_table();
                    // TODO: Figure out where this goes. Maybe have frame call palette_table when displaying.
                    frame.set_universal_background_rgb(
                        palette_table.universal_background_rgb(),
                    );

                    let column_in_tile = mem.regs_mut().current_address.x_scroll().fine();
                    let current_palette_table_index =
                        self.attribute_register.current_palette_table_index(column_in_tile);
                    let palette = palette_table.background_palette(current_palette_table_index);

                    let current_background_pixel = self.pattern_register.palette_index(column_in_tile)
                        .map_or(Rgbt::Transparent, |palette_index| Rgbt::Opaque(palette[palette_index]));

                    frame.set_background_pixel(pixel_column, pixel_row, current_background_pixel);
                }

                // TODO: Verify if this need to be delayed like self.rendering_enabled.
                if sprites_enabled {
                    let (sprite_pixel, priority, is_sprite_0) = self.oam_registers.step(&mem.palette_table());
                    frame.set_sprite_pixel(
                        pixel_column,
                        pixel_row,
                        sprite_pixel,
                        priority,
                        is_sprite_0,
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
                    mem.oam_mut().maybe_corrupt_starting_byte(oam_addr, self.clock.cycle());
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
                    return;
                }

                if self.secondary_oam.is_full() {
                    // TODO: self.all_sprites_evaluated?
                    self.oam_data_read = self.secondary_oam.read();
                } else {
                    self.secondary_oam.write(self.oam_data_read);
                }

                if !mem.regs().oam_addr.new_sprite_started() {
                    // The current sprite is in range, copy one more byte of its data over.
                    self.secondary_oam.advance();
                    self.all_sprites_evaluated = mem.regs_mut().oam_addr.next_field();
                    return;
                }

                // Check if the y coordinate is on screen.
                if let Some(pixel_row) = self.clock.scanline_pixel_row()
                    && let Some(top_sprite_row) = PixelRow::try_from_u8(self.oam_data_read)
                    && let Some(offset) = pixel_row.difference(top_sprite_row)
                    && offset < (mem.regs().sprite_height() as u8)
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

            GetSpritePatternLowByte => {
                let low_offset = 0x0;
                let (address, visible) = self.current_sprite_pattern_address(mem, low_offset);
                if mem.regs().rendering_enabled() {
                    let pattern_low = mem.read(address);
                    if visible {
                        self.oam_registers.registers[self.oam_register_index]
                            .set_pattern_low(pattern_low);
                    }
                }
            }
            GetSpritePatternHighByte => {
                let high_offset = 0x8;
                let (address, visible) = self.current_sprite_pattern_address(mem, high_offset);
                if mem.regs().rendering_enabled() {
                    let pattern_high = mem.read(address);
                    if visible {
                        self.oam_registers.registers[self.oam_register_index]
                            .set_pattern_high(pattern_high);
                    }
                }
            }
            IncrementOamRegisterIndex => {
                self.oam_register_index += 1;
            }

            StartVisibleScanlines => {
                info!(target: "ppustage", "{}\tVISIBLE SCANLINES", self.clock);
            }
            StartPostRenderScanline => {
                info!(target: "ppustage", "{}\tPOST-RENDER SCANLINE", self.clock);
            }
            StartVblankScanlines => {
                info!(target: "ppustage", "{}\tVBLANK SCANLINES", self.clock);
            }
            StartPreRenderScanline => {
                info!(target: "ppustage", "{}\tPRE-RENDER SCANLINE", self.clock);
            }

            StartReadingBackgroundTiles => {
                info!(target: "ppustage", "{}\t\tREADING BACKGROUND TILES", self.clock);
            }
            StopReadingBackgroundTiles => {
                info!(target: "ppustage", "{}\t\tENDED READING BACKGROUND TILES", self.clock);
            }

            StartClearingSecondaryOam => {
                info!(target: "ppustage", "{}\t\tCLEARING SECONDARY OAM", self.clock);
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
                    info!(target: "ppuflags", " {}\tSuppressing vblank.", self.clock);
                } else {
                    mem.regs_mut().start_vblank(&self.clock);
                }

                mem.regs_mut().suppress_vblank_active = false;
            }
            RequestNmi => {
                info!(target: "ppuflags", " {}\tNMI requested.", self.clock);
                mem.regs_mut().nmi_requested = true;
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
                mem.regs_mut().stop_vblank(&self.clock);
                mem.regs_mut().clear_sprite0_hit();
                mem.regs_mut().clear_sprite_overflow();
            }
        }
    }

    fn current_sprite_pattern_address(&self, mem: &PpuMemory, high_low_offset: u16) -> (PpuAddress, bool) {
        let sprite_table_side = mem.regs().sprite_table_side();
        let sprite_height = mem.regs().sprite_height();
        let sprite_table_side = match sprite_height {
            SpriteHeight::Normal => sprite_table_side,
            SpriteHeight::Tall => self.next_sprite_pattern_index.tall_sprite_pattern_table_side(),
        };

        let address;
        let visible;
        if let Some(pixel_row) = self.clock.scanline_pixel_row() {
            let attributes = self.oam_registers.registers[self.oam_register_index].attributes();
            if let Some((pattern_index, row_in_half, v)) = self.next_sprite_pattern_index.index_and_row(
                self.current_sprite_y,
                attributes.flip_vertically(),
                sprite_height,
                pixel_row
            ) {
                visible = v;
                address = PpuAddress::in_pattern_table(
                    sprite_table_side, pattern_index, row_in_half, high_low_offset);
            } else {
                // Sprite not on current scanline. TODO: what address should be here?
                address = PpuAddress::from_u16(0x1000);
                visible = false;
            }
        } else {
            // Pre-render scanline. TODO: use correct address based upon pattern index.
            address = PpuAddress::from_u16(0x1000);
            visible = false;
        }

        (address, visible)
    }
}

pub struct StepResult {
    pub is_last_cycle_of_frame: bool,
    pub should_generate_nmi: bool,
}

#[cfg(test)]
mod tests {
    use crate::memory::cpu::cpu_address::CpuAddress;
    use crate::memory::memory;
    use crate::ppu::register::ppu_registers::*;

    use super::*;

    #[rustfmt::skip]
    const CPU_CTRL: CpuAddress     = CpuAddress::new(0x2000);
    #[rustfmt::skip]
    const CPU_SCROLL: CpuAddress   = CpuAddress::new(0x2005);
    const CPU_PPU_ADDR: CpuAddress = CpuAddress::new(0x2006);
    const CPU_PPU_DATA: CpuAddress = CpuAddress::new(0x2007);

    const PPU_ZERO: PpuAddress = PpuAddress::ZERO;

    #[test]
    fn basic() {
        let mut ppu = Ppu::new(Clock::mesen_compatible());
        let mut mem = memory::test_data::memory();
        let mut ppu_mem = mem.as_ppu_memory();
        let mut frame = Frame::new();

        assert_eq!(ppu_mem.regs().write_toggle(), WriteToggle::FirstByte);
        ppu.step(&mut ppu_mem, &mut frame);
        assert_eq!(ppu_mem.regs().write_toggle(), WriteToggle::FirstByte);

        for i in 0x0000..0xFFFF {
            let value = ppu_mem.read(PpuAddress::from_u16(i));
            assert_eq!(value, 0);
        }
    }

    #[test]
    fn set_ppu_address() {
        let mut ppu = Ppu::new(Clock::mesen_compatible());
        let mut mem = memory::test_data::memory();
        let mut frame = Frame::new();

        mem.as_ppu_memory().regs_mut().next_address = PpuAddress::from_u16(0b0111_1111_1111_1111);

        let high_half = 0b1110_1100;
        mem.as_cpu_memory().write(CPU_PPU_ADDR, high_half);
        ppu.step(&mut mem.as_ppu_memory(), &mut frame);
        assert_eq!(mem.ppu_regs().write_toggle(), WriteToggle::SecondByte);
        assert_eq!(mem.ppu_regs().current_address, PPU_ZERO);
        assert_eq!(
            mem.ppu_regs().next_address,
            PpuAddress::from_u16(0b0010_1100_1111_1111)
        );
        assert_eq!(mem.ppu_regs().next_address.x_scroll().to_u8(), 0b1111_1000);
        assert_eq!(mem.ppu_regs().next_address.y_scroll().to_u8(), 0b0011_1010);

        println!("PPUData: {}", mem.ppu_regs().current_address);

        let low_half = 0b1010_1010;
        mem.as_cpu_memory().write(CPU_PPU_ADDR, low_half);
        ppu.step(&mut mem.as_ppu_memory(), &mut frame);
        println!("PPUData: {}", mem.ppu_regs().current_address);
        assert_eq!(mem.ppu_regs().write_toggle(), WriteToggle::FirstByte);
        assert_eq!(
            mem.ppu_regs().next_address,
            PpuAddress::from_u16(0b0010_1100_1010_1010)
        );
        assert_eq!(
            mem.ppu_regs().current_address,
            PpuAddress::from_u16(0b0010_1100_1010_1010)
        );

        let current_address = mem.ppu_regs().current_address;
        mem.as_ppu_memory().write(current_address, 184);
        let value = mem.as_cpu_memory().read(CPU_PPU_DATA).unwrap();
        ppu.step(&mut mem.as_ppu_memory(), &mut frame);
        assert_eq!(value, 0);
        assert_eq!(mem.ppu_regs().pending_ppu_data(), 184);
        let value = mem.as_cpu_memory().read(CPU_PPU_DATA).unwrap();
        ppu.step(&mut mem.as_ppu_memory(), &mut frame);
        assert_eq!(value, 184);
        assert_eq!(mem.ppu_regs().pending_ppu_data(), 0);
    }

    #[test]
    fn set_scroll() {
        let mut ppu = Ppu::new(Clock::mesen_compatible());
        let mut mem = memory::test_data::memory();
        let mut frame = Frame::new();

        mem.as_ppu_memory().regs_mut().next_address = PpuAddress::from_u16(0b0111_1111_1111_1111);

        mem.as_cpu_memory().write(CPU_CTRL, 0b1111_1101);
        ppu.step(&mut mem.as_ppu_memory(), &mut frame);
        assert_eq!(mem.ppu_regs().write_toggle(), WriteToggle::FirstByte);
        assert_eq!(
            mem.ppu_regs().next_address,
            PpuAddress::from_u16(0b0111_0111_1111_1111)
        );
        assert_eq!(mem.ppu_regs().current_address, PPU_ZERO);
        assert_eq!(mem.ppu_regs().next_address.x_scroll().to_u8(), 0b1111_1000);
        assert_eq!(mem.ppu_regs().next_address.y_scroll().to_u8(), 0b1111_1011);

        let x_scroll = 0b1100_1100;
        mem.as_cpu_memory().write(CPU_SCROLL, x_scroll);
        ppu.step(&mut mem.as_ppu_memory(), &mut frame);
        assert_eq!(mem.ppu_regs().write_toggle(), WriteToggle::SecondByte);
        assert_eq!(
            mem.ppu_regs().next_address,
            PpuAddress::from_u16(0b0111_0111_1111_1001)
        );
        assert_eq!(mem.ppu_regs().current_address, PPU_ZERO);
        assert_eq!(mem.ppu_regs().next_address.x_scroll().to_u8(), x_scroll);
        assert_eq!(mem.ppu_regs().next_address.y_scroll().to_u8(), 0b1111_1011);

        let y_scroll = 0b1010_1010;
        mem.as_cpu_memory().write(CPU_SCROLL, y_scroll);
        ppu.step(&mut mem.as_ppu_memory(), &mut frame);
        assert_eq!(mem.ppu_regs().write_toggle(), WriteToggle::FirstByte);
        assert_eq!(
            mem.ppu_regs().next_address,
            PpuAddress::from_u16(0b0010_0110_1011_1001)
        );
        assert_eq!(mem.ppu_regs().current_address, PPU_ZERO);
        assert_eq!(mem.ppu_regs().next_address.x_scroll().to_u8(), x_scroll);
        assert_eq!(mem.ppu_regs().next_address.y_scroll().to_u8(), y_scroll);

        mem.as_cpu_memory().write(CPU_CTRL, 0b0000_0010);
        ppu.step(&mut mem.as_ppu_memory(), &mut frame);
        assert_eq!(mem.ppu_regs().write_toggle(), WriteToggle::FirstByte);
        assert_eq!(
            mem.ppu_regs().next_address,
            PpuAddress::from_u16(0b0010_1010_1011_1001)
        );
        assert_eq!(mem.ppu_regs().current_address, PPU_ZERO);
        assert_eq!(mem.ppu_regs().next_address.x_scroll().to_u8(), x_scroll);
        assert_eq!(mem.ppu_regs().next_address.y_scroll().to_u8(), y_scroll);
    }

    #[test]
    fn ctrl_ppuaddr_interference() {
        let mut ppu = Ppu::new(Clock::mesen_compatible());
        let mut mem = memory::test_data::memory();
        let mut frame = Frame::new();

        mem.as_ppu_memory().regs_mut().next_address = PpuAddress::from_u16(0b0111_1111_1111_1111);

        let high_half = 0b1110_1101;
        mem.as_cpu_memory().write(CPU_PPU_ADDR, high_half);
        ppu.step(&mut mem.as_ppu_memory(), &mut frame);
        assert_eq!(mem.ppu_regs().write_toggle(), WriteToggle::SecondByte);
        assert_eq!(
            mem.ppu_regs().next_address,
            PpuAddress::from_u16(0b0010_1101_1111_1111)
        );
        assert_eq!(mem.ppu_regs().current_address, PPU_ZERO);

        mem.as_cpu_memory().write(CPU_CTRL, 0b1111_1100);
        ppu.step(&mut mem.as_ppu_memory(), &mut frame);
        assert_eq!(mem.ppu_regs().write_toggle(), WriteToggle::SecondByte);
        assert_eq!(
            mem.ppu_regs().next_address,
            PpuAddress::from_u16(0b0010_0001_1111_1111)
        );
        assert_eq!(mem.ppu_regs().current_address, PPU_ZERO);

        let low_half = 0b1010_1010;
        mem.as_cpu_memory().write(CPU_PPU_ADDR, low_half);
        ppu.step(&mut mem.as_ppu_memory(), &mut frame);
        assert_eq!(mem.ppu_regs().write_toggle(), WriteToggle::FirstByte);
        assert_eq!(
            mem.ppu_regs().next_address,
            PpuAddress::from_u16(0b0010_0001_1010_1010)
        );
        assert_eq!(
            mem.ppu_regs().current_address,
            PpuAddress::from_u16(0b0010_0001_1010_1010),
            "Bad VRAM (not temp)"
        );
        assert_eq!(mem.ppu_regs().next_address.x_scroll().to_u8(), 0b0101_0000);
        assert_eq!(mem.ppu_regs().next_address.y_scroll().to_u8(), 0b0110_1010);
    }

    #[test]
    fn scroll_ppuaddr_interference() {
        let mut ppu = Ppu::new(Clock::mesen_compatible());
        let mut mem = memory::test_data::memory();
        let mut frame = Frame::new();

        mem.as_ppu_memory().regs_mut().next_address = PpuAddress::from_u16(0b0000_1111_1110_0000);

        mem.as_cpu_memory().write(CPU_CTRL, 0b1111_1101);
        ppu.step(&mut mem.as_ppu_memory(), &mut frame);
        assert_eq!(
            mem.ppu_regs().next_address,
            PpuAddress::from_u16(0b0000_0111_1110_0000)
        );

        let x_scroll = 0b1111_1111;
        mem.as_cpu_memory().write(CPU_SCROLL, x_scroll);
        ppu.step(&mut mem.as_ppu_memory(), &mut frame);
        assert_eq!(mem.ppu_regs().write_toggle(), WriteToggle::SecondByte);
        assert_eq!(
            mem.ppu_regs().next_address,
            PpuAddress::from_u16(0b0000_0111_1111_1111)
        );
        assert_eq!(mem.ppu_regs().current_address, PPU_ZERO);
        assert_eq!(mem.ppu_regs().next_address.x_scroll().to_u8(), x_scroll);
        assert_eq!(mem.ppu_regs().next_address.y_scroll().to_u8(), 0b1111_1000);

        let low_half = 0b1010_1010;
        mem.as_cpu_memory().write(CPU_PPU_ADDR, low_half);
        ppu.step(&mut mem.as_ppu_memory(), &mut frame);
        assert_eq!(mem.ppu_regs().write_toggle(), WriteToggle::FirstByte);
        assert_eq!(
            mem.ppu_regs().next_address,
            PpuAddress::from_u16(0b0000_0111_1010_1010)
        );
        assert_eq!(
            mem.ppu_regs().current_address,
            PpuAddress::from_u16(0b0000_0111_1010_1010)
        );
        assert_eq!(mem.ppu_regs().next_address.x_scroll().to_u8(), 0b0101_0111);
        assert_eq!(mem.ppu_regs().next_address.y_scroll().to_u8(), 0b1110_1000);
    }
}
