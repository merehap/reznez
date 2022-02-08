use std::cell::RefCell;
use std::rc::Rc;

use crate::memory::memory::{Memory, PALETTE_TABLE_START};
use crate::memory::ppu_address::PpuAddress;
use crate::ppu::clock::Clock;
use crate::ppu::oam::Oam;
use crate::ppu::register::ppu_registers::*;
use crate::ppu::register::register_type::RegisterType;
use crate::ppu::register::registers::ctrl::SpriteHeight;
use crate::ppu::register::registers::ppu_data::PpuData;
use crate::ppu::register::registers::status::Status;
use crate::ppu::render::frame::Frame;

const FIRST_VBLANK_CYCLE: u64 = 3 * 27384;
const SECOND_VBLANK_CYCLE: u64 = 3 * 57165;

pub struct Ppu {
    registers: Rc<RefCell<PpuRegisters>>,
    oam: Oam,

    clock: Clock,

    address_latch: Option<u8>,
    vram_address: PpuAddress,
    vram_data: u8,

    x_scroll_offset: u8,
    y_scroll_offset: u8,

    suppress_vblank_active: bool,
    nmi_was_enabled_last_cycle: bool,
}

impl Ppu {
    pub fn new(registers: Rc<RefCell<PpuRegisters>>) -> Ppu {
        Ppu {
            registers,
            oam: Oam::new(),

            clock: Clock::new(),

            address_latch: None,

            vram_address: PpuAddress::from_u16(0),
            vram_data: 0,

            x_scroll_offset: 0,
            y_scroll_offset: 0,

            suppress_vblank_active: false,
            nmi_was_enabled_last_cycle: false,
        }
    }

    pub fn status(&self) -> Status {
        self.registers.borrow().status
    }

    pub fn clock(&self) -> &Clock {
        &self.clock
    }

    pub fn reset_address_latch(&mut self) {
        self.address_latch = None;
    }

    pub fn step(&mut self, memory: &mut Memory, frame: &mut Frame) -> StepResult {
        let total_cycles = self.clock().total_cycles();

        if self.clock.cycle() == 1 {
            self.registers.borrow_mut().maybe_decay_latch();
        }

        let latch_access = self.registers.borrow_mut().take_latch_access();
        let mut should_generate_nmi = false;
        if let Some(latch_access) = latch_access {
            should_generate_nmi = self.process_latch_access(memory, latch_access);
        }

        // TODO: Fix the first two vblank cycles to not be special-cased if possible.
        if total_cycles == FIRST_VBLANK_CYCLE || total_cycles == SECOND_VBLANK_CYCLE {
            // TODO: Why don't we have the following enabled here?
            // Maybe just need to have "= false" to end it too.
            // self.status.vblank_active = true;
            if self.can_generate_nmi() {
                should_generate_nmi = true;
            }
        } else if total_cycles < SECOND_VBLANK_CYCLE {
            // Do nothing.
        } else if self.clock.scanline() == 241 && self.clock.cycle() == 1 {
            if !self.suppress_vblank_active {
                self.registers.borrow_mut().start_vblank();
            }

            self.suppress_vblank_active = false;
            if self.can_generate_nmi() {
                should_generate_nmi = true;
            }
        } else if self.clock.scanline() == 261 && self.clock.cycle() == 1 {
            self.registers.borrow_mut().stop_vblank();
            self.registers.borrow_mut().clear_sprite0_hit();
        } else if self.clock.scanline() == 1 && self.clock.cycle() == 1 {
            if self.registers.borrow().mask.background_enabled {
                self.render_background(memory, frame);
            }

            if self.registers.borrow().sprites_enabled() {
                self.render_sprites(memory, frame);
            }
        }

        let sprite0 = self.oam.sprite0();
        // TODO: Sprite 0 hit needs lots more work.
        if self.clock.scanline() == sprite0.y_coordinate() as u16 &&
            self.clock.cycle() == 340 &&
            self.clock.cycle() > sprite0.x_coordinate() as u16 &&
            self.registers.borrow().sprites_enabled() &&
            self.registers.borrow().background_enabled() {

            self.registers.borrow_mut().set_sprite0_hit();
        }

        let oam_data = self.oam.read(self.registers.borrow().oam_addr);
        self.registers.borrow_mut().oam_data = oam_data;

        let is_palette_data = self.vram_address >= PALETTE_TABLE_START;
        // When reading palette data only, read the current data pointed to
        // by self.vram_address, not what was previously pointed to.
        let value = 
            if is_palette_data {
                memory.ppu_read(self.vram_address)
            } else {
                self.vram_data
            };
        self.registers.borrow_mut().ppu_data = PpuData {value, is_palette_data};

        let is_last_cycle_of_frame = self.clock.is_last_cycle_of_frame();
        self.clock.tick(self.rendering_enabled());

        StepResult {is_last_cycle_of_frame, should_generate_nmi}
    }

    fn process_latch_access(
        &mut self,
        memory: &mut Memory,
        latch_access: LatchAccess,
    ) -> bool {
        let value = self.registers.borrow().latch_value();
        let mut should_generate_nmi = false;

        use RegisterType::*;
        use AccessMode::*;
        match (latch_access.register_type, latch_access.access_mode) {
            (OamData, Read) => {},
            (Mask | Status | OamAddr, Write) => {},

            (Ctrl, Write) => {
                if !self.nmi_was_enabled_last_cycle {
                    // Attempt to trigger the second (or higher) NMI of this frame.
                    if self.can_generate_nmi() {
                        should_generate_nmi = true;
                    }
                }

                self.nmi_was_enabled_last_cycle = self.registers.borrow().nmi_enabled();
            },

            (Status, Read) => {
                self.stop_vblank();
                self.reset_address_latch();
            },
            (OamData, Write) => self.write_oam(value),
            (PpuAddr, Write) => self.write_partial_vram_address(value),
            (PpuData, Read) => self.update_vram_data(memory),
            (PpuData, Write) => self.write_vram(memory, value),
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
    fn render_background(&mut self, memory: &Memory, frame: &mut Frame) {
        let palette_table = memory.palette_table();
        frame.set_universal_background_rgb(palette_table.universal_background_rgb());

        let name_table_number = self.registers.borrow().name_table_number();
        //let _name_table_mirroring = memory.name_table_mirroring();
        let background_table_side = self.registers.borrow().background_table_side();
        memory.name_table(name_table_number).render(
            &memory.pattern_table(background_table_side),
            &palette_table,
            -(self.x_scroll_offset as i16),
            -(self.y_scroll_offset as i16),
            frame,
        );
        memory.name_table(name_table_number.next_horizontal()).render(
            &memory.pattern_table(background_table_side),
            &palette_table,
            -(self.x_scroll_offset as i16) + 256,
            -(self.y_scroll_offset as i16),
            frame,
        );
    }

    fn render_sprites(&mut self, memory: &Memory, frame: &mut Frame) {
        frame.clear_sprite_buffer();

        let sprite_table_side = self.registers.borrow().sprite_table_side();
        let pattern_table = memory.pattern_table(sprite_table_side);
        let palette_table = memory.palette_table();
        let sprite_height = self.registers.borrow().sprite_height();

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
                    memory.pattern_table(sprite.tall_sprite_pattern_table_side());
                sprite.render_tall(&pattern_table, &palette_table, is_sprite0, frame);
            }
        }
    }

    fn rendering_enabled(&self) -> bool {
        self.registers.borrow().sprites_enabled() ||
            self.registers.borrow().background_enabled()
    }

    fn can_generate_nmi(&self) -> bool {
        self.registers.borrow().vblank_active() &&
            self.registers.borrow().nmi_enabled()
    }

    fn write_oam(&mut self, value: u8) {
        let oam_addr = self.registers.borrow().oam_addr;
        self.oam.write(oam_addr, value);
        // Advance to next sprite byte to write.
        self.registers.borrow_mut().oam_addr = oam_addr.wrapping_add(1);
    }

    fn update_vram_data(&mut self, memory: &Memory) {
        let vram_data_source =
            if self.vram_address >= PALETTE_TABLE_START {
                // Even though palette ram isn't mirrored down, its vram data is.
                // https://forums.nesdev.org/viewtopic.php?t=18627
                self.vram_address.subtract(0x1000)
            } else {
                self.vram_address
            };
        self.vram_data = memory.ppu_read(vram_data_source);

        let increment = self.registers.borrow().vram_address_increment() as u16;
        self.vram_address = self.vram_address.advance(increment);
    }

    fn write_vram(&mut self, memory: &mut Memory, value: u8) {
        memory.ppu_write(self.vram_address, value);
        let increment = self.registers.borrow().vram_address_increment() as u16;
        self.vram_address = self.vram_address.advance(increment);
    }

    fn write_partial_vram_address(&mut self, value: u8) {
        if let Some(upper) = self.address_latch {
            self.vram_address = PpuAddress::from_u16((u16::from(upper) << 8) + u16::from(value));
            self.address_latch = None;
        } else {
            self.address_latch = Some(value);
        }
    }

    fn write_scroll_dimension(&mut self, dimension: u8) {
        if let Some(x_scroll_offset) = self.address_latch {
            self.x_scroll_offset = x_scroll_offset;
            self.y_scroll_offset = dimension;
            self.address_latch = None;
        } else {
            self.address_latch = Some(dimension);
        }
    }

    fn stop_vblank(&mut self) {
        self.registers.borrow_mut().status.vblank_active = false;
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
