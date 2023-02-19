use crate::memory::memory::PpuMemory;
use crate::memory::ppu::ppu_address::{PpuAddress, XScroll, YScroll};
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
use crate::ppu::register::registers::ppu_data::PpuData;
use crate::ppu::register::registers::attribute_register::AttributeRegister;
use crate::ppu::register::registers::pattern_register::PatternRegister;
use crate::ppu::render::frame::Frame;
use crate::ppu::sprite::sprite_attributes::SpriteAttributes;
use crate::ppu::sprite::oam::Oam;
use crate::ppu::sprite::oam_index::OamIndex;
use crate::ppu::sprite::secondary_oam::SecondaryOam;
use crate::ppu::sprite::oam_registers::OamRegisters;
use crate::ppu::sprite::sprite_y::SpriteY;
use crate::ppu::sprite::sprite_height::SpriteHeight;

pub struct Ppu {
    oam: Oam,
    oam_index: OamIndex,
    secondary_oam: SecondaryOam,
    oam_registers: OamRegisters,
    oam_register_index: usize,
    clear_oam: bool,

    clock: Clock,

    current_address: PpuAddress,
    next_address: PpuAddress,

    pub pending_data: u8,

    write_toggle: WriteToggle,

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
    pub fn new() -> Ppu {
        Ppu {
            oam: Oam::new(),
            oam_index: OamIndex::new(),
            secondary_oam: SecondaryOam::new(),
            oam_registers: OamRegisters::new(),
            oam_register_index: 0,
            clear_oam: false,

            clock: Clock::new(),

            current_address: PpuAddress::ZERO,
            next_address: PpuAddress::ZERO,

            pending_data: 0,

            write_toggle: WriteToggle::FirstByte,

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

    pub fn oam(&self) -> &Oam {
        &self.oam
    }

    pub fn clock(&self) -> &Clock {
        &self.clock
    }

    pub fn current_address(&self) -> PpuAddress {
        self.current_address
    }

    pub fn active_name_table_quadrant(&self) -> NameTableQuadrant {
        self.next_address.name_table_quadrant()
    }

    pub fn x_scroll(&self) -> XScroll {
        self.next_address.x_scroll()
    }

    pub fn y_scroll(&self) -> YScroll {
        self.next_address.y_scroll()
    }

    pub fn step(&mut self, mem: &mut PpuMemory, frame: &mut Frame) -> StepResult {
        if self.clock.cycle() == 1 {
            mem.regs_mut().maybe_decay_latch();
        }

        let latch_access = mem.regs_mut().take_latch_access();

        self.nmi_requested = false;
        if let Some(latch_access) = latch_access {
            self.nmi_requested = self.process_latch_access(mem, latch_access);
        }

        // TODO: Figure out how to eliminate duplication and the index.
        let len = self.frame_actions.current_cycle_actions(&self.clock).len();
        for i in 0..len {
            let cycle_action = self.frame_actions.current_cycle_actions(&self.clock)[i];
            self.execute_cycle_action(mem, frame, cycle_action);
        }


        let is_last_cycle_of_frame = self.clock.is_last_cycle_of_frame();
        self.clock.tick(mem.regs().rendering_enabled());
        let should_generate_nmi = self.nmi_requested && mem.regs().can_generate_nmi();

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

        let tile_column = self.current_address.x_scroll().coarse();
        let tile_row = self.current_address.y_scroll().coarse();
        let row_in_tile = self.current_address.y_scroll().fine();
        let name_table_quadrant = self.current_address.name_table_quadrant();

        let background_enabled = mem.regs().background_enabled();
        let sprites_enabled = mem.regs().sprites_enabled();
        let rendering_enabled = background_enabled || sprites_enabled;

        use CycleAction::*;
        match cycle_action {
            GetPatternIndex => {
                if !rendering_enabled { return; }
                let address = PpuAddress::in_name_table(name_table_quadrant, tile_column, tile_row);
                self.next_pattern_index = PatternIndex::new(mem.read(address));
            }
            GetPaletteIndex => {
                if !rendering_enabled { return; }
                let address = PpuAddress::in_attribute_table(name_table_quadrant, tile_column, tile_row);
                let attribute_byte = mem.read(address);
                let palette_table_index =
                    PaletteTableIndex::from_attribute_byte(attribute_byte, tile_column, tile_row);
                self.attribute_register.set_pending_palette_table_index(palette_table_index);
            }
            GetPatternLowByte => {
                if !rendering_enabled { return; }
                let address = PpuAddress::in_pattern_table(
                    background_table_side, self.next_pattern_index, row_in_tile, false);
                self.pattern_register.set_pending_low_byte(mem.read(address));
            }
            GetPatternHighByte => {
                if !rendering_enabled { return; }
                let address = PpuAddress::in_pattern_table(
                    background_table_side, self.next_pattern_index, row_in_tile, true);
                self.pattern_register.set_pending_high_byte(mem.read(address));
            }

            GotoNextTileColumn => {
                if !rendering_enabled { return; }
                self.current_address.increment_coarse_x_scroll();
            }
            GotoNextPixelRow => {
                if !rendering_enabled { return; }
                self.current_address.increment_fine_y_scroll();
            }
            ResetTileColumn => {
                if !rendering_enabled { return; }
                self.current_address.copy_x_scroll(self.next_address);
                self.current_address.copy_horizontal_name_table_side(self.next_address);
            }
            PrepareForNextTile => {
                if !rendering_enabled { return; }
                self.attribute_register.prepare_next_palette_table_index();
                self.pattern_register.load_next_palette_indexes();
            }
            SetPixel => {
                let (pixel_column, pixel_row) = PixelIndex::try_from_clock(&self.clock).unwrap().to_column_row();
                if background_enabled {
                    let palette_table = mem.palette_table();
                    // TODO: Figure out where this goes. Maybe have frame call palette_table when displaying.
                    frame.set_universal_background_rgb(
                        palette_table.universal_background_rgb(),
                    );

                    let column_in_tile = self.current_address.x_scroll().fine();
                    let current_palette_table_index =
                        self.attribute_register.current_palette_table_index(column_in_tile);
                    let palette = palette_table.background_palette(current_palette_table_index);

                    let current_background_pixel = self.pattern_register.palette_index(column_in_tile)
                        .map_or(Rgbt::Transparent, |palette_index| Rgbt::Opaque(palette[palette_index]));

                    frame.set_background_pixel(pixel_column, pixel_row, current_background_pixel);
                }

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
                if sprites_enabled && background_enabled
                    && frame.pixel(mem.regs().mask, pixel_column, pixel_row).1.hit()
                {
                    mem.regs_mut().set_sprite0_hit();
                }
            }
            PrepareForNextPixel => {
                if !background_enabled { return; }
                self.pattern_register.shift_left();
                self.attribute_register.push_next_palette_table_index();
            }

            ReadOamByte => {
                if !rendering_enabled { return; }
                // This is a dummy read if OAM clear is active. TODO: Can this be removed?
                mem.regs_mut().oam_data = self.oam.read_sprite_data(self.oam_index);
                if self.clear_oam {
                    mem.regs_mut().oam_data = 0xFF;
                }
            }
            WriteSecondaryOamByte => {
                if !rendering_enabled { return; }

                if self.clear_oam {
                    self.secondary_oam.write(mem.regs().oam_data);
                    self.secondary_oam.advance();
                    return;
                }

                if self.oam_index.end_reached() {
                    // Reading and incrementing still happen after sprite evaluation is
                    // complete, but writes fail (i.e. they don't happen).
                    // TODO: Writes failing should result in a read occuring here.
                    self.oam_index.next_sprite();
                    return;
                }

                if self.secondary_oam.is_full() {
                    // TODO: Does this go before oam_index.end_reached()?
                    mem.regs_mut().oam_data = self.secondary_oam.read();
                } else {
                    self.secondary_oam.write(mem.regs().oam_data);
                }

                if !self.oam_index.new_sprite_started() {
                    // The current sprite is in range, copy one more byte of its data over.
                    self.secondary_oam.advance();
                    self.oam_index.next_field();
                    return;
                }

                // Check if the y coordinate is on screen.
                if let Some(pixel_row) = self.clock.scanline_pixel_row()
                    && let Some(top_sprite_row) = PixelRow::try_from_u8(mem.regs().oam_data)
                    && let Some(offset) = pixel_row.difference(top_sprite_row)
                    && offset < (mem.regs().sprite_height() as u8)
                {
                    if self.oam_index.is_at_sprite_0() {
                        self.sprite_0_present = true;
                    }

                    if self.secondary_oam.is_full() {
                        mem.regs_mut().set_sprite_overflow();
                    }

                    self.secondary_oam.advance();
                    self.oam_index.next_field();
                    return;
                }

                if self.secondary_oam.is_full() {
                    // Sprite overflow hardware bug
                    // https://www.nesdev.org/wiki/PPU_sprite_evaluation#Details
                    self.oam_index.corrupt_sprite_y_index();
                }

                self.oam_index.next_sprite();
            }
            ReadSpriteY => {
                if !rendering_enabled { return; }
                self.current_sprite_y = SpriteY::new(self.secondary_oam.read_and_advance());
            }
            ReadSpritePatternIndex => {
                if !rendering_enabled { return; }
                self.next_sprite_pattern_index = PatternIndex::new(self.secondary_oam.read_and_advance());
            }
            ReadSpriteAttributes => {
                if !rendering_enabled { return; }

                let attributes = SpriteAttributes::from_u8(self.secondary_oam.read_and_advance());
                self.oam_registers.registers[self.oam_register_index].set_attributes(attributes);
            }
            ReadSpriteX => {
                if !rendering_enabled { return; }

                let x_counter = self.secondary_oam.read_and_advance();
                self.oam_registers.registers[self.oam_register_index].set_x_counter(x_counter);
            }
            DummyReadSpriteX => {
                if !rendering_enabled { return; }
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

                let pattern_low = mem.read(address);
                if rendering_enabled && visible {
                    self.oam_registers.registers[self.oam_register_index]
                        .set_pattern_low(pattern_low);
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

                let pattern_high = mem.read(address);
                if rendering_enabled {
                    if visible {
                        self.oam_registers.registers[self.oam_register_index]
                            .set_pattern_high(pattern_high);
                    }

                    // FIXME: Hack. Make a separate CycleAction for this.
                    self.oam_register_index += 1;
                }
            }

            ResetForOamClear => {
                self.secondary_oam.reset_index();
                self.clear_oam = true;
            }
            ResetForSpriteEvaluation => {
                self.secondary_oam.reset_index();
                self.clear_oam = false;
                self.oam_register_index = 0;
                self.sprite_0_present = false;
                self.oam_index.reset();
            }
            ResetForTransferToOamRegisters => {
                // TODO: Determine if this needs to occur on cycle 256 instead.
                self.secondary_oam.reset_index();
                self.oam_registers.set_sprite_0_presence(self.sprite_0_present);
            }

            StartVblank => {
                if !self.suppress_vblank_active {
                    mem.regs_mut().start_vblank();
                }

                self.suppress_vblank_active = false;
            }
            RequestNmi => {
                self.nmi_requested = true;
            }
            SetInitialScrollOffsets => {
                if !background_enabled { return; }
                self.current_address = self.next_address;
            }
            SetInitialYScroll => {
                if !background_enabled { return; }
                self.current_address.copy_y_scroll(self.next_address);
            }

            ClearFlags => {
                mem.regs_mut().stop_vblank();
                mem.regs_mut().clear_sprite0_hit();
                mem.regs_mut().clear_sprite_overflow();
            }
            UpdateOamData => {
                mem.regs_mut().oam_data = self.oam.read(mem.regs().oam_addr);
            }
        }
    }

    fn update_ppu_data(&self, mem: &mut PpuMemory) {
        let is_palette_data = self.current_address >= PpuAddress::PALETTE_TABLE_START;
        // When reading palette data only, read the current data pointed to
        // by self.current_address, not what was previously pointed to.
        let value = if is_palette_data {
            mem.read(self.current_address)
        } else {
            self.pending_data
        };
        mem.regs_mut().ppu_data = PpuData { value, is_palette_data };
    }

    fn process_latch_access(
        &mut self,
        mem: &mut PpuMemory,
        latch_access: LatchAccess,
    ) -> bool {
        let value = mem.regs().latch_value();
        let mut request_nmi = false;

        use AccessMode::*;
        use RegisterType::*;
        match (latch_access.register_type, latch_access.access_mode) {
            // 0x2000
            (Ctrl, Read) => unreachable!(),
            (Ctrl, Write) => request_nmi = self.write_ctrl(mem.regs(), value),
            // 0x2001
            (Mask, Read) => unreachable!(),
            (Mask, Write) => {}
            // 0x2002
            (Status, Read) => self.read_status(mem.regs_mut()),
            (Status, Write) => {}
            // 0x2003
            (OamAddr, Read) => unreachable!(),
            (OamAddr, Write) => {}
            // 0x2004
            (OamData, Read) => {}
            (OamData, Write) => self.write_oam_data(mem.regs_mut(), value),
            // 0x2005
            (Scroll, Read) => unreachable!(),
            (Scroll, Write) => self.write_scroll_dimension(value),
            // 0x2006
            (PpuAddr, Read) => unreachable!(),
            (PpuAddr, Write) => self.write_byte_to_next_address(value),
            // 0x2007
            (PpuData, Read) => self.update_pending_data_then_advance_current_address(mem),
            (PpuData, Write) => self.write_then_advance_current_address(mem, value),
        }

        self.update_ppu_data(mem);

        request_nmi
    }

    // Write 0x2000
    fn write_ctrl(&mut self, regs: &PpuRegisters, value: u8) -> bool {
        self.next_address.set_name_table_quadrant(NameTableQuadrant::from_last_two_bits(value));
        // Potentially attempt to trigger the second (or higher) NMI of this frame.
        let request_nmi = !self.nmi_was_enabled_last_cycle;
        self.nmi_was_enabled_last_cycle = regs.nmi_enabled();
        request_nmi
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
    fn write_oam_data(&mut self, regs: &mut PpuRegisters, value: u8) {
        let oam_addr = regs.oam_addr;
        self.oam.write(oam_addr, value);
        // Advance to next sprite byte to write.
        regs.oam_addr = oam_addr.wrapping_add(1);
    }

    // Write 0x2005
    fn write_scroll_dimension(&mut self, dimension: u8) {
        match self.write_toggle {
            WriteToggle::FirstByte => self.next_address.set_x_scroll(dimension),
            WriteToggle::SecondByte => self.next_address.set_y_scroll(dimension),
        }

        self.write_toggle.toggle();
    }

    // Write 0x2006
    fn write_byte_to_next_address(&mut self, value: u8) {
        match self.write_toggle {
            WriteToggle::FirstByte => self.next_address.set_high_byte(value),
            WriteToggle::SecondByte => {
                self.next_address.set_low_byte(value);
                self.current_address = self.next_address;
            }
        }

        self.write_toggle.toggle();
    }

    // Read 0x2007
    fn update_pending_data_then_advance_current_address(&mut self, mem: &PpuMemory) {
        self.pending_data = mem.read(self.current_address.to_pending_data_source());
        self.current_address.advance(mem.regs().current_address_increment());
    }

    // Write 0x2007
    fn write_then_advance_current_address(&mut self, mem: &mut PpuMemory, value: u8) {
        mem.write(self.current_address, value);
        self.current_address.advance(mem.regs().current_address_increment());
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
        let mut ppu = Ppu::new();
        let mut mem = memory::test_data::memory();
        let mut ppu_mem = mem.as_ppu_memory();
        let mut frame = Frame::new();

        assert_eq!(ppu.write_toggle, WriteToggle::FirstByte);
        ppu.step(&mut ppu_mem, &mut frame);
        assert_eq!(ppu.write_toggle, WriteToggle::FirstByte);

        for i in 0x0000..0xFFFF {
            let value = ppu_mem.read(PpuAddress::from_u16(i));
            assert_eq!(value, 0);
        }
    }

    #[test]
    fn set_ppu_address() {
        let mut ppu = Ppu::new();
        let mut mem = memory::test_data::memory();
        let mut frame = Frame::new();

        ppu.next_address = PpuAddress::from_u16(0b0111_1111_1111_1111);

        let high_half = 0b1110_1100;
        mem.as_cpu_memory().write(CPU_PPU_ADDR, high_half);
        ppu.step(&mut mem.as_ppu_memory(), &mut frame);
        assert_eq!(ppu.write_toggle, WriteToggle::SecondByte);
        assert_eq!(ppu.current_address, PPU_ZERO);
        assert_eq!(
            ppu.next_address,
            PpuAddress::from_u16(0b0010_1100_1111_1111)
        );
        assert_eq!(ppu.next_address.x_scroll().to_u8(), 0b1111_1000);
        assert_eq!(ppu.next_address.y_scroll().to_u8(), 0b0011_1010);

        println!("PPUData: {}", ppu.current_address);

        let low_half = 0b1010_1010;
        mem.as_cpu_memory().write(CPU_PPU_ADDR, low_half);
        ppu.step(&mut mem.as_ppu_memory(), &mut frame);
        println!("PPUData: {}", ppu.current_address);
        assert_eq!(ppu.write_toggle, WriteToggle::FirstByte);
        assert_eq!(
            ppu.next_address,
            PpuAddress::from_u16(0b0010_1100_1010_1010)
        );
        assert_eq!(
            ppu.current_address,
            PpuAddress::from_u16(0b0010_1100_1010_1010)
        );

        mem.as_ppu_memory().write(ppu.current_address, 184);
        let value = mem.as_cpu_memory().read(CPU_PPU_DATA).unwrap();
        ppu.step(&mut mem.as_ppu_memory(), &mut frame);
        assert_eq!(value, 0);
        assert_eq!(ppu.pending_data, 184);
        let value = mem.as_cpu_memory().read(CPU_PPU_DATA).unwrap();
        ppu.step(&mut mem.as_ppu_memory(), &mut frame);
        assert_eq!(value, 184);
        assert_eq!(ppu.pending_data, 0);
    }

    #[test]
    fn set_scroll() {
        let mut ppu = Ppu::new();
        let mut mem = memory::test_data::memory();
        let mut frame = Frame::new();

        ppu.next_address = PpuAddress::from_u16(0b0111_1111_1111_1111);

        mem.as_cpu_memory().write(CPU_CTRL, 0b1111_1101);
        ppu.step(&mut mem.as_ppu_memory(), &mut frame);
        assert_eq!(ppu.write_toggle, WriteToggle::FirstByte);
        assert_eq!(
            ppu.next_address,
            PpuAddress::from_u16(0b0111_0111_1111_1111)
        );
        assert_eq!(ppu.current_address, PPU_ZERO);
        assert_eq!(ppu.next_address.x_scroll().to_u8(), 0b1111_1000);
        assert_eq!(ppu.next_address.y_scroll().to_u8(), 0b1111_1011);

        let x_scroll = 0b1100_1100;
        mem.as_cpu_memory().write(CPU_SCROLL, x_scroll);
        ppu.step(&mut mem.as_ppu_memory(), &mut frame);
        assert_eq!(ppu.write_toggle, WriteToggle::SecondByte);
        assert_eq!(
            ppu.next_address,
            PpuAddress::from_u16(0b0111_0111_1111_1001)
        );
        assert_eq!(ppu.current_address, PPU_ZERO);
        assert_eq!(ppu.next_address.x_scroll().to_u8(), x_scroll);
        assert_eq!(ppu.next_address.y_scroll().to_u8(), 0b1111_1011);

        let y_scroll = 0b1010_1010;
        mem.as_cpu_memory().write(CPU_SCROLL, y_scroll);
        ppu.step(&mut mem.as_ppu_memory(), &mut frame);
        assert_eq!(ppu.write_toggle, WriteToggle::FirstByte);
        assert_eq!(
            ppu.next_address,
            PpuAddress::from_u16(0b0010_0110_1011_1001)
        );
        assert_eq!(ppu.current_address, PPU_ZERO);
        assert_eq!(ppu.next_address.x_scroll().to_u8(), x_scroll);
        assert_eq!(ppu.next_address.y_scroll().to_u8(), y_scroll);

        mem.as_cpu_memory().write(CPU_CTRL, 0b0000_0010);
        ppu.step(&mut mem.as_ppu_memory(), &mut frame);
        assert_eq!(ppu.write_toggle, WriteToggle::FirstByte);
        assert_eq!(
            ppu.next_address,
            PpuAddress::from_u16(0b0010_1010_1011_1001)
        );
        assert_eq!(ppu.current_address, PPU_ZERO);
        assert_eq!(ppu.next_address.x_scroll().to_u8(), x_scroll);
        assert_eq!(ppu.next_address.y_scroll().to_u8(), y_scroll);
    }

    #[test]
    fn ctrl_ppuaddr_interference() {
        let mut ppu = Ppu::new();
        let mut mem = memory::test_data::memory();
        let mut frame = Frame::new();

        ppu.next_address = PpuAddress::from_u16(0b0111_1111_1111_1111);

        let high_half = 0b1110_1101;
        mem.as_cpu_memory().write(CPU_PPU_ADDR, high_half);
        ppu.step(&mut mem.as_ppu_memory(), &mut frame);
        assert_eq!(ppu.write_toggle, WriteToggle::SecondByte);
        assert_eq!(
            ppu.next_address,
            PpuAddress::from_u16(0b0010_1101_1111_1111)
        );
        assert_eq!(ppu.current_address, PPU_ZERO);

        mem.as_cpu_memory().write(CPU_CTRL, 0b1111_1100);
        ppu.step(&mut mem.as_ppu_memory(), &mut frame);
        assert_eq!(ppu.write_toggle, WriteToggle::SecondByte);
        assert_eq!(
            ppu.next_address,
            PpuAddress::from_u16(0b0010_0001_1111_1111)
        );
        assert_eq!(ppu.current_address, PPU_ZERO);

        let low_half = 0b1010_1010;
        mem.as_cpu_memory().write(CPU_PPU_ADDR, low_half);
        ppu.step(&mut mem.as_ppu_memory(), &mut frame);
        assert_eq!(ppu.write_toggle, WriteToggle::FirstByte);
        assert_eq!(
            ppu.next_address,
            PpuAddress::from_u16(0b0010_0001_1010_1010)
        );
        assert_eq!(
            ppu.current_address,
            PpuAddress::from_u16(0b0010_0001_1010_1010),
            "Bad VRAM (not temp)"
        );
        assert_eq!(ppu.next_address.x_scroll().to_u8(), 0b0101_0000);
        assert_eq!(ppu.next_address.y_scroll().to_u8(), 0b0110_1010);
    }

    #[test]
    fn scroll_ppuaddr_interference() {
        let mut ppu = Ppu::new();
        let mut mem = memory::test_data::memory();
        let mut frame = Frame::new();

        ppu.next_address = PpuAddress::from_u16(0b0000_1111_1110_0000);

        mem.as_cpu_memory().write(CPU_CTRL, 0b1111_1101);
        ppu.step(&mut mem.as_ppu_memory(), &mut frame);
        assert_eq!(
            ppu.next_address,
            PpuAddress::from_u16(0b0000_0111_1110_0000)
        );

        let x_scroll = 0b1111_1111;
        mem.as_cpu_memory().write(CPU_SCROLL, x_scroll);
        ppu.step(&mut mem.as_ppu_memory(), &mut frame);
        assert_eq!(ppu.write_toggle, WriteToggle::SecondByte);
        assert_eq!(
            ppu.next_address,
            PpuAddress::from_u16(0b0000_0111_1111_1111)
        );
        assert_eq!(ppu.current_address, PPU_ZERO);
        assert_eq!(ppu.next_address.x_scroll().to_u8(), x_scroll);
        assert_eq!(ppu.next_address.y_scroll().to_u8(), 0b1111_1000);

        let low_half = 0b1010_1010;
        mem.as_cpu_memory().write(CPU_PPU_ADDR, low_half);
        ppu.step(&mut mem.as_ppu_memory(), &mut frame);
        assert_eq!(ppu.write_toggle, WriteToggle::FirstByte);
        assert_eq!(
            ppu.next_address,
            PpuAddress::from_u16(0b0000_0111_1010_1010)
        );
        assert_eq!(
            ppu.current_address,
            PpuAddress::from_u16(0b0000_0111_1010_1010)
        );
        assert_eq!(ppu.next_address.x_scroll().to_u8(), 0b0101_0111);
        assert_eq!(ppu.next_address.y_scroll().to_u8(), 0b1110_1000);
    }
}
