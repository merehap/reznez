use log::info;

use crate::memory::memory::PpuMemory;
use crate::memory::ppu::ppu_address::PpuAddress;
use crate::ppu::clock::Clock;
use crate::ppu::cycle_action::cycle_action::CycleAction;
use crate::ppu::cycle_action::frame_actions::{FrameActions, NTSC_FRAME_ACTIONS};
use crate::ppu::name_table::name_table_quadrant::NameTableQuadrant;
use crate::ppu::palette::palette_table_index::PaletteTableIndex;
use crate::ppu::palette::rgbt::Rgbt;
use crate::ppu::pattern_table::PatternIndex;
use crate::ppu::pixel_index::{PixelIndex, PixelRow};
use crate::ppu::register::ppu_registers::*;
use crate::ppu::register::register_type::RegisterType;
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

    write_toggle: WriteToggle,

    rendering_enabled: bool,
    toggle_rendering_enabled: bool,

    suppress_vblank_active: bool,
    nmi_requested: bool,
    nmi_was_enabled_last_cycle: bool,

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

            write_toggle: WriteToggle::FirstByte,

            rendering_enabled: false,
            toggle_rendering_enabled: false,

            suppress_vblank_active: false,
            nmi_requested: false,
            nmi_was_enabled_last_cycle: false,

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
        let is_last_cycle_of_frame = self.clock.tick(self.rendering_enabled);

        if self.toggle_rendering_enabled {
            self.rendering_enabled = !self.rendering_enabled;
            self.toggle_rendering_enabled = false;
        }

        //println!("PPUCYCLE: {}", self.clock.cycle());
        if self.clock.cycle() == 1 {
            mem.regs_mut().maybe_decay_ppu_io_bus();
        }

        let latch_access = mem.regs_mut().take_io_bus_access();

        self.nmi_requested = false;
        if let Some(latch_access) = latch_access {
            self.nmi_requested = self.process_latch_access(mem, latch_access);
        }

        // TODO: Figure out how to eliminate duplication and the index.
        let len = self.frame_actions.current_cycle_actions(&self.clock).len();
        for i in 0..len {
            let cycle_action = self.frame_actions.current_cycle_actions(&self.clock)[i];
            info!(target: "ppusteps", "\t{:?}", cycle_action);
            self.execute_cycle_action(mem, frame, cycle_action);
        }

        let should_generate_nmi = self.nmi_requested && mem.regs().can_generate_nmi();

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
        let sprite_table_side = mem.regs().sprite_table_side();

        let tile_column = mem.regs().current_address.x_scroll().coarse();
        let tile_row = mem.regs().current_address.y_scroll().coarse();
        let row_in_tile = mem.regs().current_address.y_scroll().fine();
        let name_table_quadrant = mem.regs().current_address.name_table_quadrant();

        let background_enabled = mem.regs().background_enabled();
        let sprites_enabled = mem.regs().sprites_enabled();

        use CycleAction::*;
        match cycle_action {
            GetPatternIndex => {
                if !self.rendering_enabled { return; }
                let address = PpuAddress::in_name_table(name_table_quadrant, tile_column, tile_row);
                self.next_pattern_index = PatternIndex::new(mem.read(address, true));
            }
            GetPaletteIndex => {
                if !self.rendering_enabled { return; }
                let address = PpuAddress::in_attribute_table(name_table_quadrant, tile_column, tile_row);
                let attribute_byte = mem.read(address, true);
                let palette_table_index =
                    PaletteTableIndex::from_attribute_byte(attribute_byte, tile_column, tile_row);
                self.attribute_register.set_pending_palette_table_index(palette_table_index);
            }
            GetPatternLowByte => {
                if !self.rendering_enabled { return; }
                let address = PpuAddress::in_pattern_table(
                    background_table_side, self.next_pattern_index, row_in_tile, false);
                self.pattern_register.set_pending_low_byte(mem.read(address, true));
            }
            GetPatternHighByte => {
                if !self.rendering_enabled { return; }
                let address = PpuAddress::in_pattern_table(
                    background_table_side, self.next_pattern_index, row_in_tile, true);
                self.pattern_register.set_pending_high_byte(mem.read(address, true));
            }

            GotoNextTileColumn => {
                if !self.rendering_enabled { return; }
                mem.regs_mut().current_address.increment_coarse_x_scroll();
            }
            GotoNextPixelRow => {
                if !self.rendering_enabled { return; }
                mem.regs_mut().current_address.increment_fine_y_scroll();
            }
            ResetTileColumn => {
                if !self.rendering_enabled { return; }
                let next_address = mem.regs().next_address;
                mem.regs_mut().current_address.copy_x_scroll(next_address);
                mem.regs_mut().current_address.copy_horizontal_name_table_side(next_address);
            }
            PrepareForNextTile => {
                if !self.rendering_enabled { return; }
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

            ResetOamAddress => {
                if !self.rendering_enabled { return; }
                mem.regs_mut().oam_addr.reset();
            }

            ReadOamByte => {
                if !self.rendering_enabled { return; }
                // This is a dummy read if OAM clear is active. TODO: Can this be removed?
                self.oam_data_read = mem.oam().peek(mem.regs().oam_addr);
                if self.clear_oam {
                    self.oam_data_read = 0xFF;
                }
            }
            WriteSecondaryOamByte => {
                if !self.rendering_enabled { return; }

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
                if !self.rendering_enabled { return; }
                self.current_sprite_y = SpriteY::new(self.secondary_oam.read_and_advance());
            }
            ReadSpritePatternIndex => {
                if !self.rendering_enabled { return; }
                self.next_sprite_pattern_index = PatternIndex::new(self.secondary_oam.read_and_advance());
            }
            ReadSpriteAttributes => {
                if !self.rendering_enabled { return; }

                let attributes = SpriteAttributes::from_u8(self.secondary_oam.read_and_advance());
                self.oam_registers.registers[self.oam_register_index].set_attributes(attributes);
            }
            ReadSpriteX => {
                if !self.rendering_enabled { return; }

                let x_counter = self.secondary_oam.read_and_advance();
                self.oam_registers.registers[self.oam_register_index].set_x_counter(x_counter);
            }
            DummyReadSpriteX => {
                // TODO
                //if !self.rendering_enabled { return; }
            }

            GetSpritePatternLowByte => {
                // FIXME: Hack
                let mut address = PpuAddress::from_u16(0x1000);
                let mut visible = false;
                if let Some(pixel_row) = self.clock.scanline_pixel_row() {
                    let attributes = self.oam_registers.registers[self.oam_register_index].attributes();
                    let sprite_height = mem.regs().sprite_height();
                    if let Some((pattern_index, row_in_half)) = self.next_sprite_pattern_index.index_and_row(
                        self.current_sprite_y,
                        attributes.flip_vertically(),
                        sprite_height,
                        pixel_row
                    ) {
                        visible = true;
                        let sprite_table_side = match sprite_height  {
                            SpriteHeight::Normal => sprite_table_side,
                            SpriteHeight::Tall => self.next_sprite_pattern_index.tall_sprite_pattern_table_side(),
                        };

                        address = PpuAddress::in_pattern_table(
                            sprite_table_side, pattern_index, row_in_half, false);
                    }
                }

                if self.rendering_enabled {
                    let pattern_low = mem.read(address, true);
                    if visible {
                        self.oam_registers.registers[self.oam_register_index]
                            .set_pattern_low(pattern_low);
                    }
                }
            }
            GetSpritePatternHighByte => {
                // FIXME: Hack
                let mut address = PpuAddress::from_u16(0x1000);
                let mut visible = false;
                if let Some(pixel_row) = self.clock.scanline_pixel_row() {
                    let attributes = self.oam_registers.registers[self.oam_register_index].attributes();
                    let sprite_height = mem.regs().sprite_height();
                    if let Some((pattern_index, row_in_half)) = self.next_sprite_pattern_index.index_and_row(
                        self.current_sprite_y,
                        attributes.flip_vertically(),
                        sprite_height,
                        pixel_row
                    ) {
                        visible = true;

                        let sprite_table_side = match sprite_height  {
                            SpriteHeight::Normal => sprite_table_side,
                            SpriteHeight::Tall => self.next_sprite_pattern_index.tall_sprite_pattern_table_side(),
                        };

                        address = PpuAddress::in_pattern_table(
                            sprite_table_side, pattern_index, row_in_half, true);
                    }
                }

                if self.rendering_enabled {
                    let pattern_high = mem.read(address, true);
                    if visible {
                        self.oam_registers.registers[self.oam_register_index]
                            .set_pattern_high(pattern_high);
                    }

                    // FIXME: Hack. Make a separate CycleAction for this.
                    self.oam_register_index += 1;
                }
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
                if !self.suppress_vblank_active {
                    mem.regs_mut().start_vblank();
                } else {
                    info!(target: "ppuflags", "\tSuppressing vblank.");
                }

                self.suppress_vblank_active = false;
            }
            RequestNmi => {
                info!(target: "ppuflags", "\tNMI requested.");
                self.nmi_requested = true;
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
                mem.regs_mut().stop_vblank();
                mem.regs_mut().clear_sprite0_hit();
                mem.regs_mut().clear_sprite_overflow();
            }
        }
    }

    fn process_latch_access(
        &mut self,
        mem: &mut PpuMemory,
        latch_access: LatchAccess,
    ) -> bool {
        let value = mem.regs().ppu_io_bus_value();
        let mut request_nmi = false;

        use AccessMode::*;
        use RegisterType::*;
        match (latch_access.register_type, latch_access.access_mode) {
            // 0x2000
            (Ctrl, Read) => unreachable!(),
            (Ctrl, Write) => request_nmi = self.write_ctrl(mem.regs_mut(), value),
            // 0x2001
            (Mask, Read) => unreachable!(),
            (Mask, Write) => self.write_mask(mem.regs()),
            // 0x2002
            (Status, Read) => self.read_status(mem.regs_mut()),
            (Status, Write) => {}
            // 0x2003
            (OamAddr, Read) => unreachable!(),
            (OamAddr, Write) => {}
            // 0x2004
            (OamData, Read) => {}
            (OamData, Write) => self.write_oam_data(mem, value),
            // 0x2005
            (Scroll, Read) => unreachable!(),
            (Scroll, Write) => self.write_scroll_dimension(mem.regs_mut(), value),
            // 0x2006
            (PpuAddr, Read) => unreachable!(),
            (PpuAddr, Write) => self.write_ppu_address(mem, value),
            // 0x2007
            (PpuData, Read) => self.read_ppu_data(mem),
            (PpuData, Write) => {},
        }

        request_nmi
    }

    // Write 0x2000
    fn write_ctrl(&mut self, regs: &mut PpuRegisters, value: u8) -> bool {
        regs.next_address.set_name_table_quadrant(NameTableQuadrant::from_last_two_bits(value));
        // Potentially attempt to trigger the second (or higher) NMI of this frame.
        let request_nmi = !self.nmi_was_enabled_last_cycle;
        self.nmi_was_enabled_last_cycle = regs.nmi_enabled();
        request_nmi
    }

    // Write 0x2001
    fn write_mask(&mut self, regs: &PpuRegisters) {
        if self.rendering_enabled != regs.rendering_enabled() {
            self.toggle_rendering_enabled = true;
        }
    }

    // Read 0x2002
    fn read_status(&mut self, regs: &mut PpuRegisters) {
        regs.stop_vblank();
        // https://wiki.nesdev.org/w/index.php?title=NMI#Race_condition
        if self.clock.scanline() == 241 && self.clock.cycle() == 1 {
            self.suppress_vblank_active = true;
        }

        self.write_toggle = WriteToggle::FirstByte;
    }

    // Write 0x2003
    fn write_oam_data(&mut self, mem: &mut PpuMemory, value: u8) {
        let oam_addr = mem.regs().oam_addr;
        mem.oam_mut().write(oam_addr, value);
        // Advance to next sprite byte to write.
        mem.regs_mut().oam_addr.increment();
    }

    // Write 0x2005
    fn write_scroll_dimension(&mut self, regs: &mut PpuRegisters, dimension: u8) {
        match self.write_toggle {
            WriteToggle::FirstByte => regs.next_address.set_x_scroll(dimension),
            WriteToggle::SecondByte => regs.next_address.set_y_scroll(dimension),
        }

        self.write_toggle.toggle();
    }

    // Write 0x2006
    fn write_ppu_address(&mut self, mem: &mut PpuMemory, value: u8) {
        match self.write_toggle {
            WriteToggle::FirstByte => mem.regs_mut().next_address.set_high_byte(value),
            WriteToggle::SecondByte => {
                mem.regs_mut().next_address.set_low_byte(value);
                mem.regs_mut().current_address = mem.regs().next_address;
                mem.process_current_ppu_address(mem.regs().current_address);
            }
        }

        self.write_toggle.toggle();
    }

    fn read_ppu_data(&self, mem: &mut PpuMemory) {
        mem.process_current_ppu_address(mem.regs().current_address);
    }
}

pub struct StepResult {
    pub is_last_cycle_of_frame: bool,
    pub should_generate_nmi: bool,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
enum WriteToggle {
    FirstByte,
    SecondByte,
}

impl WriteToggle {
    fn toggle(&mut self) {
        use WriteToggle::*;
        *self = match self {
            FirstByte => SecondByte,
            SecondByte => FirstByte,
        };
    }
}

#[cfg(test)]
mod tests {
    use crate::memory::cpu::cpu_address::CpuAddress;
    use crate::memory::memory;

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

        assert_eq!(ppu.write_toggle, WriteToggle::FirstByte);
        ppu.step(&mut ppu_mem, &mut frame);
        assert_eq!(ppu.write_toggle, WriteToggle::FirstByte);

        for i in 0x0000..0xFFFF {
            let value = ppu_mem.read(PpuAddress::from_u16(i), false);
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
        assert_eq!(ppu.write_toggle, WriteToggle::SecondByte);
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
        assert_eq!(ppu.write_toggle, WriteToggle::FirstByte);
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
        assert_eq!(mem.ppu_regs().pending_ppu_data, 184);
        let value = mem.as_cpu_memory().read(CPU_PPU_DATA).unwrap();
        ppu.step(&mut mem.as_ppu_memory(), &mut frame);
        assert_eq!(value, 184);
        assert_eq!(mem.ppu_regs().pending_ppu_data, 0);
    }

    #[test]
    fn set_scroll() {
        let mut ppu = Ppu::new(Clock::mesen_compatible());
        let mut mem = memory::test_data::memory();
        let mut frame = Frame::new();

        mem.as_ppu_memory().regs_mut().next_address = PpuAddress::from_u16(0b0111_1111_1111_1111);

        mem.as_cpu_memory().write(CPU_CTRL, 0b1111_1101);
        ppu.step(&mut mem.as_ppu_memory(), &mut frame);
        assert_eq!(ppu.write_toggle, WriteToggle::FirstByte);
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
        assert_eq!(ppu.write_toggle, WriteToggle::SecondByte);
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
        assert_eq!(ppu.write_toggle, WriteToggle::FirstByte);
        assert_eq!(
            mem.ppu_regs().next_address,
            PpuAddress::from_u16(0b0010_0110_1011_1001)
        );
        assert_eq!(mem.ppu_regs().current_address, PPU_ZERO);
        assert_eq!(mem.ppu_regs().next_address.x_scroll().to_u8(), x_scroll);
        assert_eq!(mem.ppu_regs().next_address.y_scroll().to_u8(), y_scroll);

        mem.as_cpu_memory().write(CPU_CTRL, 0b0000_0010);
        ppu.step(&mut mem.as_ppu_memory(), &mut frame);
        assert_eq!(ppu.write_toggle, WriteToggle::FirstByte);
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
        assert_eq!(ppu.write_toggle, WriteToggle::SecondByte);
        assert_eq!(
            mem.ppu_regs().next_address,
            PpuAddress::from_u16(0b0010_1101_1111_1111)
        );
        assert_eq!(mem.ppu_regs().current_address, PPU_ZERO);

        mem.as_cpu_memory().write(CPU_CTRL, 0b1111_1100);
        ppu.step(&mut mem.as_ppu_memory(), &mut frame);
        assert_eq!(ppu.write_toggle, WriteToggle::SecondByte);
        assert_eq!(
            mem.ppu_regs().next_address,
            PpuAddress::from_u16(0b0010_0001_1111_1111)
        );
        assert_eq!(mem.ppu_regs().current_address, PPU_ZERO);

        let low_half = 0b1010_1010;
        mem.as_cpu_memory().write(CPU_PPU_ADDR, low_half);
        ppu.step(&mut mem.as_ppu_memory(), &mut frame);
        assert_eq!(ppu.write_toggle, WriteToggle::FirstByte);
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
        assert_eq!(ppu.write_toggle, WriteToggle::SecondByte);
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
        assert_eq!(ppu.write_toggle, WriteToggle::FirstByte);
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
