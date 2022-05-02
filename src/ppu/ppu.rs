use crate::memory::memory::{PpuMemory, PALETTE_TABLE_START};
use crate::memory::ppu::ppu_address::{PpuAddress, XScroll, YScroll};
use crate::ppu::clock::Clock;
use crate::ppu::name_table::name_table::Rectangle;
use crate::ppu::name_table::name_table_quadrant::NameTableQuadrant;
use crate::ppu::oam::Oam;
use crate::ppu::palette::palette_table_index::PaletteTableIndex;
use crate::ppu::palette::rgbt::Rgbt;
use crate::ppu::pattern_table::{PatternIndex, PatternRegister};
use crate::ppu::pixel_index::{ColumnInTile, PixelColumn, PixelRow, RowInTile};
use crate::ppu::register::ppu_registers::*;
use crate::ppu::register::register_type::RegisterType;
use crate::ppu::register::registers::ppu_data::PpuData;
use crate::ppu::render::frame::Frame;

pub struct Ppu {
    oam: Oam,

    clock: Clock,

    current_address: PpuAddress,
    next_address: PpuAddress,

    pending_data: u8,

    write_toggle: WriteToggle,

    suppress_vblank_active: bool,
    nmi_was_enabled_last_cycle: bool,

    current_background_sliver: [Rgbt; 8],
    pending_pattern_register: PatternRegister,
}

impl Ppu {
    pub fn new() -> Ppu {
        Ppu {
            oam: Oam::new(),

            clock: Clock::new(),

            current_address: PpuAddress::from_u16(0),
            next_address: PpuAddress::from_u16(0),

            pending_data: 0,

            write_toggle: WriteToggle::FirstByte,

            suppress_vblank_active: false,
            nmi_was_enabled_last_cycle: false,

            current_background_sliver: [Rgbt::Transparent; 8],
            pending_pattern_register: PatternRegister::empty(),
        }
    }

    pub fn oam(&self) -> &Oam {
        &self.oam
    }

    pub fn clock(&self) -> &Clock {
        &self.clock
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
        let mut maybe_generate_nmi = false;
        if let Some(latch_access) = latch_access {
            maybe_generate_nmi = self.process_latch_access(mem, latch_access);
        }

        let scanline = self.clock.scanline();
        let cycle = self.clock.cycle();
        if let Some(pixel_row) = PixelRow::try_from_u16(scanline) {
            if cycle == 1 {
                self.maybe_render_scanline(pixel_row, mem, frame);
            }

            if (1..=256).contains(&cycle) {
                let pixel_column = PixelColumn::try_from_u16(cycle - 1).unwrap();
                let (pattern_index, palette_table_index, _column_in_tile, row_in_tile) =
                    self.tile_entry_for_pixel(pixel_column, pixel_row, mem);

                let background_table_side = mem.regs().background_table_side();
                match cycle % 8 {
                    2 => { /* Name table */ }
                    4 => { /* Attribute table */ }
                    6 => self.pending_pattern_register.add_low_byte(
                        &mem.pattern_table(background_table_side),
                        pattern_index,
                        row_in_tile,
                    ),
                    0 => {
                        self.pending_pattern_register.add_high_byte(
                            &mem.pattern_table(background_table_side),
                            pattern_index,
                            row_in_tile,
                        );
                        let palette =
                            mem.palette_table().background_palette(palette_table_index);
                        self.pending_pattern_register.take_tile_sliver(
                            palette,
                            &mut self.current_background_sliver,
                        );
                    }
                    _ => { /* Only even cycles commit changes for two-cycle fetches. */ }
                }

                self.maybe_set_sprite0_hit(mem, frame);
            }
        }

        match (self.clock.scanline(), self.clock.cycle()) {
            (241, 1) => {
                if !self.suppress_vblank_active {
                    mem.regs_mut().start_vblank();
                }

                self.suppress_vblank_active = false;
            }
            (241, 3) => maybe_generate_nmi = true,
            (261, 1) => {
                mem.regs_mut().stop_vblank();
                mem.regs_mut().clear_sprite0_hit();
            }
            (_, _) => { /* Do nothing. */ }
        }

        self.update_oam_data(mem.regs_mut());
        self.update_ppu_data(mem);

        let is_last_cycle_of_frame = self.clock.is_last_cycle_of_frame();
        self.clock.tick(mem.regs().rendering_enabled());
        let should_generate_nmi = maybe_generate_nmi && mem.regs().can_generate_nmi();

        StepResult { is_last_cycle_of_frame, should_generate_nmi }
    }

    fn process_latch_access(
        &mut self,
        mem: &mut PpuMemory,
        latch_access: LatchAccess,
    ) -> bool {
        let value = mem.regs().latch_value();
        let mut maybe_generate_nmi = false;

        use AccessMode::*;
        use RegisterType::*;
        match (latch_access.register_type, latch_access.access_mode) {
            (OamData, Read) => {}
            (Mask | Status | OamAddr, Write) => {}

            (Ctrl, Write) => {
                self.next_address.set_name_table_quadrant(value);
                if !self.nmi_was_enabled_last_cycle {
                    // Attempt to trigger the second (or higher) NMI of this frame.
                    maybe_generate_nmi = true;
                }

                self.nmi_was_enabled_last_cycle = mem.regs().nmi_enabled();
            }

            (Status, Read) => {
                mem.regs_mut().stop_vblank();
                // https://wiki.nesdev.org/w/index.php?title=NMI#Race_condition
                if self.clock.scanline() == 241 && self.clock.cycle() == 1 {
                    self.suppress_vblank_active = true;
                }

                self.write_toggle = WriteToggle::FirstByte;
            }
            (OamData, Write) => self.write_oam(mem.regs_mut(), value),
            (PpuAddr, Write) => self.write_byte_to_next_address(value),
            (PpuData, Read) => self.update_pending_data_then_advance_current_address(mem),
            (PpuData, Write) => self.write_then_advance_current_address(mem, value),
            (Scroll, Write) => self.write_scroll_dimension(value),

            (Ctrl | Mask | OamAddr | Scroll | PpuAddr, Read) => unreachable!(
                "The data latch should not be filled by a read to {:?}.",
                latch_access.register_type,
            ),
        }

        maybe_generate_nmi
    }

    fn maybe_render_scanline(
        &mut self,
        pixel_row: PixelRow,
        mem: &PpuMemory,
        frame: &mut Frame,
    ) {
        if mem.regs().background_enabled() {
            self.render_background_scanline(pixel_row, mem, frame);
        }

        if mem.regs().sprites_enabled() {
            self.oam.render_scanline(pixel_row, mem, frame);
        }
    }

    fn render_background_scanline(
        &self,
        pixel_row: PixelRow,
        mem: &PpuMemory,
        frame: &mut Frame,
    ) {
        let palette_table = mem.palette_table();
        frame.set_universal_background_rgb(palette_table.universal_background_rgb());

        let name_table_quadrant = self.next_address.name_table_quadrant();
        let background_table_side = mem.regs().background_table_side();
        let x_scroll = self.next_address.x_scroll();
        let y_scroll = self.next_address.y_scroll();

        let x_divider = 255 - x_scroll.to_u8();
        let y_divider = 239 - (y_scroll.to_u8() % 240);

        if let Some(bounds) = Rectangle::from_raw((0, 0), (x_divider, y_divider)) {
            mem.name_table(name_table_quadrant).render_scanline(
                pixel_row,
                &mem.pattern_table(background_table_side),
                &palette_table,
                x_scroll,
                y_scroll,
                bounds,
                frame,
            );
        }

        if !x_scroll.is_zero() {
            if let Some(bounds) =
                Rectangle::from_raw((x_divider + 1, 0), (255, y_divider))
            {
                mem.name_table(name_table_quadrant.next_horizontal())
                    .render_scanline(
                        pixel_row,
                        &mem.pattern_table(background_table_side),
                        &palette_table,
                        x_scroll,
                        y_scroll,
                        bounds,
                        frame,
                    );
            }
        }

        if !y_scroll.is_zero() {
            if let Some(bounds) =
                Rectangle::from_raw((0, y_divider + 1), (x_divider, 239))
            {
                mem.name_table(name_table_quadrant.next_vertical())
                    .render_scanline(
                        pixel_row,
                        &mem.pattern_table(background_table_side),
                        &palette_table,
                        x_scroll,
                        y_scroll.shift_down(),
                        bounds,
                        frame,
                    );
            }
        }

        if !x_scroll.is_zero() && !y_scroll.is_zero() {
            if let Some(bounds) =
                Rectangle::from_raw((x_divider + 1, y_divider + 1), (255, 239))
            {
                let position = name_table_quadrant.next_horizontal().next_vertical();
                mem.name_table(position).render_scanline(
                    pixel_row,
                    &mem.pattern_table(background_table_side),
                    &palette_table,
                    x_scroll,
                    y_scroll.shift_down(),
                    bounds,
                    frame,
                );
            }
        }
    }

    fn tile_entry_for_pixel(
        &self,
        pixel_column: PixelColumn,
        pixel_row: PixelRow,
        mem: &PpuMemory,
    ) -> (PatternIndex, PaletteTableIndex, ColumnInTile, RowInTile) {
        let mut name_table_quadrant = self.next_address.name_table_quadrant();

        let x_scroll = self.next_address.x_scroll();
        let x_divider = 255 - x_scroll.to_u8();
        if pixel_column.to_u8() > x_divider {
            name_table_quadrant = name_table_quadrant.next_horizontal();
        }

        let mut y_scroll = self.next_address.y_scroll();
        let y_divider = 239 - (y_scroll.to_u8() % 240);
        if pixel_row.to_u8() > y_divider {
            name_table_quadrant = name_table_quadrant.next_vertical();
            y_scroll = y_scroll.shift_down();
        }

        mem.name_table(name_table_quadrant).tile_entry_for_pixel(
            pixel_column,
            pixel_row,
            x_scroll,
            y_scroll,
        )
    }

    // https://wiki.nesdev.org/w/index.php?title=PPU_OAM#Sprite_zero_hits
    fn maybe_set_sprite0_hit(&self, mem: &mut PpuMemory, frame: &mut Frame) {
        let maybe_x = PixelColumn::try_from_u16(self.clock.cycle() - 1);
        let maybe_y = PixelRow::try_from_u16(self.clock.scanline());

        if let (Some(x), Some(y)) = (maybe_x, maybe_y) {
            if mem.regs().sprites_enabled()
                && mem.regs().background_enabled()
                && frame.pixel(mem.regs().mask, x, y).1.hit()
            {
                mem.regs_mut().set_sprite0_hit();
            }
        }
    }

    fn update_oam_data(&self, regs: &mut PpuRegisters) {
        let oam_data = self.oam.read(regs.oam_addr);
        regs.oam_data = oam_data;
    }

    fn update_ppu_data(&self, mem: &mut PpuMemory) {
        let is_palette_data = self.current_address >= PALETTE_TABLE_START;
        // When reading palette data only, read the current data pointed to
        // by self.current_address, not what was previously pointed to.
        let value = if is_palette_data {
            mem.read(self.current_address)
        } else {
            self.pending_data
        };
        mem.regs_mut().ppu_data = PpuData { value, is_palette_data };
    }

    fn write_oam(&mut self, regs: &mut PpuRegisters, value: u8) {
        let oam_addr = regs.oam_addr;
        self.oam.write(oam_addr, value);
        // Advance to next sprite byte to write.
        regs.oam_addr = oam_addr.wrapping_add(1);
    }

    fn update_pending_data_then_advance_current_address(&mut self, mem: &PpuMemory) {
        let mut data_source = self.current_address;
        if data_source >= PALETTE_TABLE_START {
            // Even though palette ram isn't mirrored down, its data address is.
            // https://forums.nesdev.org/viewtopic.php?t=18627
            data_source.subtract(0x1000);
        }

        self.pending_data = mem.read(data_source);

        let increment = mem.regs().current_address_increment() as u16;
        self.current_address.advance(increment);
    }

    fn write_then_advance_current_address(&mut self, mem: &mut PpuMemory, value: u8) {
        mem.write(self.current_address, value);
        let increment = mem.regs().current_address_increment() as u16;
        self.current_address.advance(increment);
    }

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

    fn write_scroll_dimension(&mut self, dimension: u8) {
        match self.write_toggle {
            WriteToggle::FirstByte => self.next_address.set_x_scroll(dimension),
            WriteToggle::SecondByte => self.next_address.set_y_scroll(dimension),
        }

        self.write_toggle.toggle();
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

    const PPU_ZERO: PpuAddress = PpuAddress::from_u16(0x0000);

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
        let value = mem.as_cpu_memory().read(CPU_PPU_DATA);
        ppu.step(&mut mem.as_ppu_memory(), &mut frame);
        assert_eq!(value, 0);
        assert_eq!(ppu.pending_data, 184);
        let value = mem.as_cpu_memory().read(CPU_PPU_DATA);
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
