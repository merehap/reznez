use crate::memory::memory::{PpuMemory, PALETTE_TABLE_START};
use crate::memory::ppu_address::PpuAddress;
use crate::ppu::clock::Clock;
use crate::ppu::oam::Oam;
use crate::ppu::register::ppu_registers::*;
use crate::ppu::register::register_type::RegisterType;
use crate::ppu::register::registers::ctrl::SpriteHeight;
use crate::ppu::register::registers::ppu_data::PpuData;
use crate::ppu::render::frame::Frame;

const FIRST_VBLANK_CYCLE: u64 = 3 * 27384;
const SECOND_VBLANK_CYCLE: u64 = 3 * 57165;

pub struct Ppu {
    oam: Oam,

    clock: Clock,

    write_toggle: WriteToggle,

    current_address: PpuAddress,
    // Can't be a PpuAddress since PpuAddress enforces bits[1] = 0 .
    next_address: u16,

    pending_data: u8,

    x_scroll_offset: u8,
    y_scroll_offset: u8,

    suppress_vblank_active: bool,
    nmi_was_enabled_last_cycle: bool,
}

impl Ppu {
    pub fn new() -> Ppu {
        Ppu {
            oam: Oam::new(),

            clock: Clock::new(),

            write_toggle: WriteToggle::FirstByte,

            current_address: PpuAddress::from_u16(0),
            next_address: 0,
            pending_data: 0,

            x_scroll_offset: 0,
            y_scroll_offset: 0,

            suppress_vblank_active: false,
            nmi_was_enabled_last_cycle: false,
        }
    }

    pub fn clock(&self) -> &Clock {
        &self.clock
    }

    pub fn step(&mut self, mem: &mut PpuMemory, frame: &mut Frame) -> StepResult {
        let total_cycles = self.clock().total_cycles();

        if self.clock.cycle() == 1 {
            mem.registers_mut().maybe_decay_latch();
        }

        let latch_access = mem.registers_mut().take_latch_access();
        let mut should_generate_nmi = false;
        if let Some(latch_access) = latch_access {
            should_generate_nmi = self.process_latch_access(mem, latch_access);
        }

        // TODO: Fix the first two vblank cycles to not be special-cased if possible.
        if total_cycles == FIRST_VBLANK_CYCLE || total_cycles == SECOND_VBLANK_CYCLE {
            // TODO: Why don't we have the following enabled here?
            // Maybe just need to have "= false" to end it too.
            // self.status.vblank_active = true;
            if mem.registers().can_generate_nmi() {
                should_generate_nmi = true;
            }
        } else if total_cycles < SECOND_VBLANK_CYCLE {
            // Do nothing.
        } else if self.clock.scanline() == 241 && self.clock.cycle() == 1 {
            if !self.suppress_vblank_active {
                mem.registers_mut().start_vblank();
            }

            self.suppress_vblank_active = false;
            if mem.registers().can_generate_nmi() {
                should_generate_nmi = true;
            }
        } else if self.clock.scanline() == 261 && self.clock.cycle() == 1 {
            mem.registers_mut().stop_vblank();
            mem.registers_mut().clear_sprite0_hit();
        } else if self.clock.scanline() == 1 && self.clock.cycle() == 65 {
            if mem.registers().mask.background_enabled {
                self.render_background(mem, frame);
            }

            if mem.registers().sprites_enabled() {
                self.render_sprites(mem, frame);
            }
        }

        let sprite0 = self.oam.sprite0();
        // TODO: Sprite 0 hit needs lots more work.
        if self.clock.scanline() == sprite0.y_coordinate() as u16 &&
            self.clock.cycle() == 340 &&
            self.clock.cycle() > sprite0.x_coordinate() as u16 &&
            mem.registers().sprites_enabled() &&
            mem.registers().background_enabled() {

            mem.registers_mut().set_sprite0_hit();
        }

        let oam_data = self.oam.read(mem.registers().oam_addr);
        mem.registers_mut().oam_data = oam_data;

        let is_palette_data = self.current_address >= PALETTE_TABLE_START;
        // When reading palette data only, read the current data pointed to
        // by self.current_address, not what was previously pointed to.
        let value = 
            if is_palette_data {
                mem.read(self.current_address)
            } else {
                self.pending_data
            };
        mem.registers_mut().ppu_data = PpuData {value, is_palette_data};

        let is_last_cycle_of_frame = self.clock.is_last_cycle_of_frame();
        self.clock.tick(mem.registers().rendering_enabled());

        StepResult {is_last_cycle_of_frame, should_generate_nmi}
    }

    fn process_latch_access(
        &mut self, mem: &mut PpuMemory, latch_access: LatchAccess,
    ) -> bool {
        let value = mem.registers().latch_value();
        let mut should_generate_nmi = false;

        use RegisterType::*;
        use AccessMode::*;
        match (latch_access.register_type, latch_access.access_mode) {
            (OamData, Read) => {},
            (Mask | Status | OamAddr, Write) => {},

            (Ctrl, Write) => {
                self.next_address &= 0b1111_0011_1111_1111;
                self.next_address |= (value as u16 & 0b0000_0011) << 10;
                if !self.nmi_was_enabled_last_cycle {
                    // Attempt to trigger the second (or higher) NMI of this frame.
                    if mem.registers().can_generate_nmi() {
                        should_generate_nmi = true;
                    }
                }

                self.nmi_was_enabled_last_cycle = mem.registers().nmi_enabled();
            },

            (Status, Read) => {
                self.stop_vblank(mem.registers_mut());
                self.write_toggle = WriteToggle::FirstByte;
            },
            (OamData, Write) => self.write_oam(mem.registers_mut(), value),
            (PpuAddr, Write) => self.partially_prepare_next_address(value),
            (PpuData, Read) => self.update_pending_data_then_advance(mem),
            (PpuData, Write) => self.write_then_advance(mem, value),
            (Scroll, Write) => self.write_scroll_dimension(value),

            (Ctrl | Mask | OamAddr | Scroll | PpuAddr, Read) =>
                unreachable!(
                    "The data latch should not be filled by a read to {:?}.",
                    latch_access.register_type,
                ),
        }

        should_generate_nmi
    }

    // FIXME: Stop rendering off-screen pixels.
    fn render_background(&mut self, mem: &PpuMemory, frame: &mut Frame) {
        let palette_table = mem.palette_table();
        frame.set_universal_background_rgb(palette_table.universal_background_rgb());

        let name_table_number = mem.registers().name_table_number();
        //let _name_table_mirroring = mem.name_table_mirroring();
        let background_table_side = mem.registers().background_table_side();
        mem.name_table(name_table_number).render(
            &mem.pattern_table(background_table_side),
            &palette_table,
            -(self.x_scroll_offset as i16),
            -(self.y_scroll_offset as i16),
            frame,
        );
        mem.name_table(name_table_number.next_horizontal()).render(
            &mem.pattern_table(background_table_side),
            &palette_table,
            -(self.x_scroll_offset as i16) + 256,
            -(self.y_scroll_offset as i16),
            frame,
        );
    }

    fn render_sprites(&mut self, mem: &PpuMemory, frame: &mut Frame) {
        frame.clear_sprite_buffer();

        let sprite_table_side = mem.registers().sprite_table_side();
        let pattern_table = mem.pattern_table(sprite_table_side);
        let palette_table = mem.palette_table();
        let sprite_height = mem.registers().sprite_height();

        // FIXME: No more sprites will be found once the end of OAM is reached,
        // effectively hiding any sprites before OAM[OAMADDR].
        let sprites = self.oam.sprites();
        // Lower index sprites are drawn on top of higher index sprites.
        for i in (0..sprites.len()).rev() {
            let is_sprite0 = i == 0;
            if sprite_height == SpriteHeight::Normal {
                sprites[i].render_normal_height(&pattern_table, &palette_table, is_sprite0, frame);
            } else {
                let sprite = sprites[i];
                let pattern_table =
                    mem.pattern_table(sprite.tall_sprite_pattern_table_side());
                sprite.render_tall(&pattern_table, &palette_table, is_sprite0, frame);
            }
        }
    }

    fn write_oam(&mut self, regs: &mut PpuRegisters, value: u8) {
        let oam_addr = regs.oam_addr;
        self.oam.write(oam_addr, value);
        // Advance to next sprite byte to write.
        regs.oam_addr = oam_addr.wrapping_add(1);
    }

    fn update_pending_data_then_advance(&mut self, mem: &PpuMemory) {
        let mut data_source = self.current_address;
        if data_source >= PALETTE_TABLE_START {
            // Even though palette ram isn't mirrored down, its data address is.
            // https://forums.nesdev.org/viewtopic.php?t=18627
            data_source = data_source.subtract(0x1000)
        }

        self.pending_data = mem.read(data_source);

        let increment = mem.registers().current_address_increment() as u16;
        self.current_address = self.current_address.advance(increment);
    }

    fn write_then_advance(&mut self, mem: &mut PpuMemory, value: u8) {
        mem.write(self.current_address, value);
        let increment = mem.registers().current_address_increment() as u16;
        self.current_address = self.current_address.advance(increment);
    }

    fn partially_prepare_next_address(&mut self, value: u8) {
        match self.write_toggle {
            WriteToggle::FirstByte => {
                self.next_address &= 0b0000_0000_1111_1111;
                self.next_address |= ((value & 0b0011_1111) as u16) << 8;
            },
            WriteToggle::SecondByte => {
                self.next_address &= 0b0111_1111_0000_0000;
                self.next_address |= value as u16;
                self.current_address = PpuAddress::from_u16(self.next_address);
            }
        }

        self.write_toggle.toggle();
    }

    /*
     * 0123456789ABCDEF: bit pos.
     *            01234  $SCROLL#1
     *  567--01234-----  $SCROLL#2
     *     67----------  $CTRL
     */
    fn write_scroll_dimension(&mut self, dimension: u8) {
        match self.write_toggle {
            WriteToggle::FirstByte => {
                self.x_scroll_offset = dimension;
                self.next_address &= 0b0111_1111_1110_0000;
                self.next_address |= (self.x_scroll_offset >> 3) as u16;
            },
            WriteToggle::SecondByte => {
                self.y_scroll_offset = dimension;
                self.next_address &= 0b0000_1100_0001_1111;
                self.next_address |= (self.y_scroll_offset as u16 >> 3) << 5;
                self.next_address |= (self.y_scroll_offset as u16 & 0b0000_0111) << 12;
            }
        }

        self.write_toggle.toggle();
    }

    fn stop_vblank(&mut self, regs: &mut PpuRegisters) {
        regs.status.vblank_active = false;
        // https://wiki.nesdev.org/w/index.php?title=NMI#Race_condition
        if self.clock.scanline() == 241 && self.clock.cycle() == 1 {
            self.suppress_vblank_active = true;
        }
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
        match self {
            WriteToggle::FirstByte => *self = WriteToggle::SecondByte,
            WriteToggle::SecondByte => *self = WriteToggle::FirstByte,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::memory::memory;
    use crate::memory::cpu_address::CpuAddress;

    use super::*;

    const CPU_CTRL:     CpuAddress = CpuAddress::new(0x2000);
    const CPU_SCROLL:   CpuAddress = CpuAddress::new(0x2005);
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

        ppu.next_address = 0b0111_1111_1111_1111;

        let high_half = 0b1110_1100;
        mem.as_cpu_memory().write(CPU_PPU_ADDR, high_half);
        ppu.step(&mut mem.as_ppu_memory(), &mut frame);
        assert_eq!(ppu.write_toggle, WriteToggle::SecondByte);
        assert_eq!(ppu.current_address, PPU_ZERO);
        assert_eq!(ppu.next_address, 0b0010_1100_1111_1111);
        assert_eq!(ppu.x_scroll_offset, 0);
        assert_eq!(ppu.y_scroll_offset, 0);

        let low_half = 0b1010_1010;
        mem.as_cpu_memory().write(CPU_PPU_ADDR, low_half);
        ppu.step(&mut mem.as_ppu_memory(), &mut frame);
        assert_eq!(ppu.write_toggle, WriteToggle::FirstByte);
        assert_eq!(ppu.next_address, 0b0010_1100_1010_1010);
        assert_eq!(ppu.current_address, PpuAddress::from_u16(0b0010_1100_1010_1010));
        assert_eq!(ppu.x_scroll_offset, 0);
        assert_eq!(ppu.y_scroll_offset, 0);

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

        ppu.next_address = 0b0111_1111_1111_1111;

        mem.as_cpu_memory().write(CPU_CTRL, 0b1111_1101);
        ppu.step(&mut mem.as_ppu_memory(), &mut frame);
        assert_eq!(ppu.write_toggle, WriteToggle::FirstByte);
        assert_eq!(ppu.next_address, 0b0111_0111_1111_1111);
        assert_eq!(ppu.current_address, PPU_ZERO);
        assert_eq!(ppu.x_scroll_offset, 0);
        assert_eq!(ppu.y_scroll_offset, 0);

        let x_scroll = 0b1100_1100;
        mem.as_cpu_memory().write(CPU_SCROLL, x_scroll);
        ppu.step(&mut mem.as_ppu_memory(), &mut frame);
        assert_eq!(ppu.write_toggle, WriteToggle::SecondByte);
        assert_eq!(ppu.next_address, 0b0111_0111_1111_1001);
        assert_eq!(ppu.current_address, PPU_ZERO);
        assert_eq!(ppu.x_scroll_offset, x_scroll);
        assert_eq!(ppu.y_scroll_offset, 0);

        let y_scroll = 0b1010_1010;
        mem.as_cpu_memory().write(CPU_SCROLL, y_scroll);
        ppu.step(&mut mem.as_ppu_memory(), &mut frame);
        assert_eq!(ppu.write_toggle, WriteToggle::FirstByte);
        assert_eq!(ppu.next_address, 0b0010_0110_1011_1001);
        assert_eq!(ppu.current_address, PPU_ZERO);
        assert_eq!(ppu.x_scroll_offset, x_scroll);
        assert_eq!(ppu.y_scroll_offset, y_scroll);

        mem.as_cpu_memory().write(CPU_CTRL, 0b0000_0010);
        ppu.step(&mut mem.as_ppu_memory(), &mut frame);
        assert_eq!(ppu.write_toggle, WriteToggle::FirstByte);
        assert_eq!(ppu.next_address, 0b0010_1010_1011_1001);
        assert_eq!(ppu.current_address, PPU_ZERO);
        assert_eq!(ppu.x_scroll_offset, x_scroll);
        assert_eq!(ppu.y_scroll_offset, y_scroll);
    }

    #[test]
    fn ctrl_ppuaddr_interference() {
        let mut ppu = Ppu::new();
        let mut mem = memory::test_data::memory();
        let mut frame = Frame::new();

        ppu.next_address = 0b0111_1111_1111_1111;

        let high_half = 0b1110_1101;
        mem.as_cpu_memory().write(CPU_PPU_ADDR, high_half);
        ppu.step(&mut mem.as_ppu_memory(), &mut frame);
        assert_eq!(ppu.write_toggle, WriteToggle::SecondByte);
        assert_eq!(ppu.next_address, 0b0010_1101_1111_1111);
        assert_eq!(ppu.current_address, PPU_ZERO);

        mem.as_cpu_memory().write(CPU_CTRL, 0b1111_1100);
        ppu.step(&mut mem.as_ppu_memory(), &mut frame);
        assert_eq!(ppu.write_toggle, WriteToggle::SecondByte);
        assert_eq!(ppu.next_address, 0b0010_0001_1111_1111);
        assert_eq!(ppu.current_address, PPU_ZERO);

        let low_half = 0b1010_1010;
        mem.as_cpu_memory().write(CPU_PPU_ADDR, low_half);
        ppu.step(&mut mem.as_ppu_memory(), &mut frame);
        assert_eq!(ppu.write_toggle, WriteToggle::FirstByte);
        assert_eq!(ppu.next_address, 0b0010_0001_1010_1010);
        assert_eq!(ppu.current_address, PpuAddress::from_u16(0b0010_0001_1010_1010), "Bad VRAM (not temp)");
        assert_eq!(ppu.x_scroll_offset, 0);
        assert_eq!(ppu.y_scroll_offset, 0);
    }

    #[test]
    fn scroll_ppuaddr_interference() {
        let mut ppu = Ppu::new();
        let mut mem = memory::test_data::memory();
        let mut frame = Frame::new();

        ppu.next_address = 0b0000_1111_1110_0000;

        mem.as_cpu_memory().write(CPU_CTRL, 0b1111_1101);
        ppu.step(&mut mem.as_ppu_memory(), &mut frame);
        assert_eq!(ppu.next_address, 0b0000_0111_1110_0000);

        let x_scroll = 0b1111_1111;
        mem.as_cpu_memory().write(CPU_SCROLL, x_scroll);
        ppu.step(&mut mem.as_ppu_memory(), &mut frame);
        assert_eq!(ppu.write_toggle, WriteToggle::SecondByte);
        assert_eq!(ppu.next_address, 0b0000_0111_1111_1111);
        assert_eq!(ppu.current_address, PPU_ZERO);
        assert_eq!(ppu.x_scroll_offset, x_scroll);
        assert_eq!(ppu.y_scroll_offset, 0);

        let low_half = 0b1010_1010;
        mem.as_cpu_memory().write(CPU_PPU_ADDR, low_half);
        ppu.step(&mut mem.as_ppu_memory(), &mut frame);
        assert_eq!(ppu.write_toggle, WriteToggle::FirstByte);
        assert_eq!(ppu.next_address, 0b0000_0111_1010_1010);
        assert_eq!(ppu.current_address, PpuAddress::from_u16(0b0000_0111_1010_1010));
        assert_eq!(ppu.x_scroll_offset, x_scroll);
        assert_eq!(ppu.y_scroll_offset, 0);
    }
}
