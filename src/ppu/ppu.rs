use std::collections::VecDeque;
use std::ops::{Index, IndexMut};

use crate::memory::memory::PpuMemory;
use crate::memory::ppu::ppu_address::{PpuAddress, XScroll, YScroll};
use crate::ppu::clock::Clock;
use crate::ppu::name_table::name_table_quadrant::NameTableQuadrant;
use crate::ppu::oam::Oam;
use crate::ppu::palette::palette_index::PaletteIndex;
use crate::ppu::palette::palette_table_index::PaletteTableIndex;
use crate::ppu::palette::rgbt::Rgbt;
use crate::ppu::pattern_table::PatternIndex;
use crate::ppu::pixel_index::{PixelIndex, ColumnInTile, PixelColumn, PixelRow};
use crate::ppu::register::ppu_registers::*;
use crate::ppu::register::register_type::RegisterType;
use crate::ppu::register::registers::ppu_data::PpuData;
use crate::ppu::render::frame::Frame;
use crate::util::bit_util::unpack_bools;

#[derive(Clone, Copy, Debug)]
pub enum CycleAction {
    GetPatternIndex,
    GetPaletteIndex,
    GetBackgroundTileLowByte,
    GetBackgroundTileHighByte,

    GotoNextTileColumn,
    GotoNextPixelRow,
    PrepareNextTile,
    ResetTileColumn,
}

pub struct Ppu {
    oam: Oam,

    clock: Clock,

    current_address: PpuAddress,
    next_address: PpuAddress,

    pending_data: u8,

    write_toggle: WriteToggle,

    suppress_vblank_active: bool,
    nmi_was_enabled_last_cycle: bool,

    current_background_pixel: Rgbt,

    next_pattern_index: PatternIndex,
    pattern_register: PatternRegister,
    attribute_register: AttributeRegister,

    background_scanline_actions: [Vec<CycleAction>; 341],
}

impl Ppu {
    pub fn new() -> Ppu {
        use CycleAction::*;
        let mut acts = Vec::new();
        // Cycle 0 (Skipped on odd, rendering frames.)
        acts.push(vec![]);
        // Cycle 1
        acts.push(vec![]);
        // Cycles 2-249: Retrieve the remaining 31 tiles used for the current scanline.
        for _ in 2..=32 {
            acts.push(vec![GetPatternIndex]);
            acts.push(vec![]);
            acts.push(vec![GetPaletteIndex]);
            acts.push(vec![]);
            acts.push(vec![GetBackgroundTileLowByte]);
            acts.push(vec![]);
            acts.push(vec![GetBackgroundTileHighByte, GotoNextTileColumn]);
            acts.push(vec![PrepareNextTile]);
        }

        // Cycles 250-257: Retrieve an unused tile then prepare for the next pixel row.
        acts.push(vec![GetPatternIndex]);
        acts.push(vec![]);
        acts.push(vec![GetPaletteIndex]);
        acts.push(vec![]);
        acts.push(vec![GetBackgroundTileLowByte]);
        acts.push(vec![]);
        acts.push(vec![GetBackgroundTileHighByte, GotoNextPixelRow]);
        acts.push(vec![ResetTileColumn, PrepareNextTile]);

        // TODO: Sprite rendering.
        for _cycle in 258..=321 {
            acts.push(vec![]);
        }

        // Cycles 322-337: Retrieve the first two tiles for the next scanline.
        for _tile in 0..=1 {
            acts.push(vec![GetPatternIndex]);
            acts.push(vec![]);
            acts.push(vec![GetPaletteIndex]);
            acts.push(vec![]);
            acts.push(vec![GetBackgroundTileLowByte]);
            acts.push(vec![]);
            acts.push(vec![GetBackgroundTileHighByte, GotoNextTileColumn]);
            acts.push(vec![PrepareNextTile]);
        }

        // Unused fetches from the Name Table.
        acts.push(vec![GetPatternIndex]);
        acts.push(vec![]);
        acts.push(vec![GetPatternIndex]);

        Ppu {
            oam: Oam::new(),

            clock: Clock::new(),

            current_address: PpuAddress::ZERO,
            next_address: PpuAddress::ZERO,

            pending_data: 0,

            write_toggle: WriteToggle::FirstByte,

            suppress_vblank_active: false,
            nmi_was_enabled_last_cycle: false,

            current_background_pixel: Rgbt::Transparent,

            next_pattern_index: PatternIndex::new(0),
            pattern_register: PatternRegister::new(),
            attribute_register: AttributeRegister::new(),

            background_scanline_actions: acts.try_into().unwrap(),
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
        let scanline = self.clock.scanline();
        let cycle = self.clock.cycle();

        if self.clock.cycle() == 1 {
            mem.regs_mut().maybe_decay_latch();
        }

        let latch_access = mem.regs_mut().take_latch_access();
        let mut maybe_generate_nmi = false;
        if let Some(latch_access) = latch_access {
            maybe_generate_nmi = self.process_latch_access(mem, latch_access);
        }

        if mem.regs().background_enabled() && ((0..=239).contains(&scanline) || scanline == 261) {
            if scanline == 261 && cycle == 320 {
                self.current_address = self.next_address;
            }

            for action in self.background_scanline_actions[usize::from(cycle)].clone() {
                self.execute_cycle_action(mem, action);
            }

            match cycle {
                322..=336 => {
                    self.pattern_register.shift_left();
                    self.attribute_register.push_next_palette_table_index();
                }
                _ => {}
            }

            if scanline == 261 && cycle >= 280 && cycle <= 304 {
                self.current_address.copy_y_scroll(self.next_address);
            }
        }

        if let Some(pixel_index) = PixelIndex::try_from_clock(&self.clock) {
            let (pixel_column, pixel_row) = pixel_index.to_column_row();
            if mem.regs().background_enabled() {
                let palette_table = mem.palette_table();
                frame.set_universal_background_rgb(
                    palette_table.universal_background_rgb(),
                );

                let column_in_tile = self.current_address.x_scroll().fine();
                let palette = palette_table.background_palette(self.attribute_register.current_palette_table_index(column_in_tile));
                self.current_background_pixel = self.pattern_register.palette_index(column_in_tile)
                    .map(|palette_index| Rgbt::Opaque(palette[palette_index]))
                    .unwrap_or(Rgbt::Transparent);

                frame.set_background_pixel(
                    pixel_column,
                    pixel_row,
                    self.current_background_pixel,
                );

                self.pattern_register.shift_left();
                self.attribute_register.push_next_palette_table_index();
            }

            let cycle = self.clock.cycle();
            if cycle == 1 {
                if mem.regs().sprites_enabled() {
                    self.oam.render_scanline(pixel_row, mem, frame);
                }
            }

            self.maybe_set_sprite0_hit(mem, frame);
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

    pub fn execute_cycle_action(&mut self, mem: &mut PpuMemory, cycle_action: CycleAction) {
        let background_table_side = mem.regs().background_table_side();
        let pattern_table = mem.pattern_table(background_table_side);
        let tile_column = self.current_address.x_scroll().coarse();
        let tile_row = self.current_address.y_scroll().coarse();
        let row_in_tile = self.current_address.y_scroll().fine();
        let name_table = mem.name_table(self.current_address.name_table_quadrant());

        use CycleAction::*;
        match cycle_action {
            GetPatternIndex => self.next_pattern_index = name_table.pattern_index(tile_column, tile_row),
            GetPaletteIndex => {
                let palette_table_index = name_table.attribute_table().palette_table_index(tile_column, tile_row);
                self.attribute_register.set_pending_palette_table_index(palette_table_index);
            }
            GetBackgroundTileLowByte => {
                let low_byte = pattern_table.read_low_byte(self.next_pattern_index, row_in_tile);
                self.pattern_register.set_pending_low_byte(low_byte);
            }
            GetBackgroundTileHighByte => {
                let high_byte = pattern_table.read_high_byte(self.next_pattern_index, row_in_tile);
                self.pattern_register.set_pending_high_byte(high_byte);
            }
            GotoNextTileColumn => self.current_address.increment_coarse_x_scroll(),
            GotoNextPixelRow => self.current_address.increment_fine_y_scroll(),
            ResetTileColumn => {
                self.current_address.copy_x_scroll(self.next_address);
                self.current_address.copy_horizontal_name_table_side(self.next_address);
            }
            PrepareNextTile => {
                self.attribute_register.prepare_next_palette_table_index();
                self.pattern_register.load_next_palette_indexes();
            }
        }
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
                self.next_address.set_name_table_quadrant(NameTableQuadrant::from_last_two_bits(value));
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

    fn write_oam(&mut self, regs: &mut PpuRegisters, value: u8) {
        let oam_addr = regs.oam_addr;
        self.oam.write(oam_addr, value);
        // Advance to next sprite byte to write.
        regs.oam_addr = oam_addr.wrapping_add(1);
    }

    fn update_pending_data_then_advance_current_address(&mut self, mem: &PpuMemory) {
        self.pending_data = mem.read(self.current_address.to_pending_data_source());
        self.current_address.advance(mem.regs().current_address_increment());
    }

    fn write_then_advance_current_address(&mut self, mem: &mut PpuMemory, value: u8) {
        mem.write(self.current_address, value);
        self.current_address.advance(mem.regs().current_address_increment());
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

pub struct PatternRegister {
    pending_low_byte: u8,
    pending_high_byte: u8,
    current_indexes: ShiftArray<Option<PaletteIndex>, 16>,
}

impl PatternRegister {
    pub fn new() -> PatternRegister {
        PatternRegister {
            pending_low_byte: 0,
            pending_high_byte: 0,
            current_indexes: ShiftArray::new(),
        }
    }

    pub fn set_pending_low_byte(&mut self, low_byte: u8) {
        self.pending_low_byte = low_byte;
    }

    pub fn set_pending_high_byte(&mut self, high_byte: u8) {
        self.pending_high_byte = high_byte;
    }

    pub fn load_next_palette_indexes(&mut self) {
        let low_bits = unpack_bools(self.pending_low_byte);
        let high_bits = unpack_bools(self.pending_high_byte);
        for i in 0..8 {
            let palette_index = match (low_bits[i], high_bits[i]) {
                (false, false) => None,
                (true , false) => Some(PaletteIndex::One),
                (false, true ) => Some(PaletteIndex::Two),
                (true , true ) => Some(PaletteIndex::Three),
            };

            self.current_indexes[i + 8] = palette_index;
        }
    }

    pub fn shift_left(&mut self) {
        self.current_indexes.shift_left();
    }

    pub fn palette_index(&self, column_in_tile: ColumnInTile) -> Option<PaletteIndex> {
        self.current_indexes[column_in_tile]
    }
}

pub struct AttributeRegister {
    pending_index: PaletteTableIndex,
    next_index: PaletteTableIndex,
    current_indexes: ShiftArray<PaletteTableIndex, 8>,
}

impl AttributeRegister {
    pub fn new() -> AttributeRegister {
        AttributeRegister {
            pending_index: PaletteTableIndex::Zero,
            next_index: PaletteTableIndex::Zero,
            current_indexes: ShiftArray::new(),
        }
    }

    pub fn set_pending_palette_table_index(&mut self, index: PaletteTableIndex) {
        self.pending_index = index;
    }

    pub fn prepare_next_palette_table_index(&mut self) {
        self.next_index = self.pending_index;
    }

    pub fn push_next_palette_table_index(&mut self) {
        self.current_indexes.push(self.next_index);
    }

    pub fn current_palette_table_index(&self, column_in_tile: ColumnInTile) -> PaletteTableIndex {
        self.current_indexes[column_in_tile]
    }
}

struct ShiftArray<T, const N: usize>(VecDeque<T>);

impl <T: Copy + Default, const N: usize> ShiftArray<T, N> {
    pub fn new() -> ShiftArray<T, N> {
        ShiftArray(VecDeque::from_iter([Default::default(); N]))
    }

    pub fn shift_left(&mut self) {
        self.0.pop_front();
        self.0.push_back(Default::default());
    }

    pub fn push(&mut self, value: T) {
        self.0.pop_front();
        self.0.push_back(value);
    }
}

impl <T, const N: usize> Index<ColumnInTile> for ShiftArray<T, N> {
    type Output = T;

    // Indexes greater than 7 are intentionally inaccessible.
    fn index(&self, column_in_tile: ColumnInTile) -> &T {
        &self.0[column_in_tile as usize]
    }
}

impl <T, const N: usize> Index<usize> for ShiftArray<T, N> {
    type Output = T;

    // Indexes greater than 7 are intentionally inaccessible.
    fn index(&self, index: usize) -> &T {
        &self.0[index]
    }
}

impl <T, const N: usize> IndexMut<usize> for ShiftArray<T, N> {
    // Indexes greater than 7 are intentionally inaccessible.
    fn index_mut(&mut self, index: usize) -> &mut T {
        &mut self.0[index]
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
