use std::cell::RefCell;
use std::rc::Rc;

use crate::memory::memory::{Memory, PALETTE_TABLE_START};
use crate::memory::ppu_address::PpuAddress;
use crate::ppu::clock::Clock;
use crate::ppu::oam::Oam;
use crate::ppu::registers::status::Status;
use crate::ppu::ppu_registers::{PpuRegisters, RegisterType, DataLatch, AccessMode};
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

    should_generate_nmi: bool,
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

            should_generate_nmi: false,
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

    pub fn oam_address(&self) -> u8 {
        self.registers.borrow().oam_addr
    }

    pub fn overwrite_oam(&mut self, oam_address: u8, value: u8) {
        self.oam[oam_address] = value;
    }

    pub fn reset_address_latch(&mut self) {
        self.address_latch = None;
    }

    pub fn should_generate_nmi(&self) -> bool {
        self.should_generate_nmi
    }

    pub fn step(&mut self, memory: &mut Memory, frame: &mut Frame) {
        let total_cycles = self.clock().total_cycles();
        self.should_generate_nmi = false;

        let latch = self.registers.borrow().latch().clone();
        if let Some(latch) = latch {
            self.process_latch(memory, latch);
            self.registers.borrow_mut().reset_latch();
        }

        // TODO: Fix the first and second vblank cycles to not be special-cased if possible.
        if total_cycles == FIRST_VBLANK_CYCLE || total_cycles == SECOND_VBLANK_CYCLE {
            // TODO: Why don't we have the following enabled here?
            // Maybe just need to have "= false" to end it too.
            // self.status.vblank_active = true;
            if self.can_generate_nmi() {
                self.should_generate_nmi = true;
            }
        } else if total_cycles < SECOND_VBLANK_CYCLE {
            // Do nothing.
        } else if self.clock.scanline() == 241 && self.clock.cycle() == 1 {
            if !self.suppress_vblank_active {
                self.registers.borrow_mut().status.vblank_active = true;
            }

            self.suppress_vblank_active = false;
            if self.can_generate_nmi() {
                self.should_generate_nmi = true;
            }
        } else if self.clock.scanline() == 261 && self.clock.cycle() == 1 {
            self.registers.borrow_mut().status.vblank_active = false;
            self.registers.borrow_mut().status.sprite0_hit = false;
        } else if self.clock.scanline() == 1 && self.clock.cycle() == 1 {
            if self.registers.borrow().mask.background_enabled {
                self.render_background(&memory, frame);
            }

            if self.registers.borrow().mask.sprites_enabled {
                self.render_sprites(&memory, frame);
            }
        }

        let sprite_0 = self.oam.sprite_0();
        // TODO: Sprite 0 hit needs lots more work.
        if self.clock.scanline() == sprite_0.y_coordinate() as u16 &&
            self.clock.cycle() == 340 &&
            self.clock.cycle() > sprite_0.x_coordinate() as u16 &&
            self.registers.borrow().mask.sprites_enabled &&
            self.registers.borrow().mask.background_enabled {

            self.registers.borrow_mut().status.sprite0_hit = true;
        }

        let oam_data = self.oam[self.registers.borrow().oam_addr];
        self.registers.borrow_mut().oam_data = oam_data;

        // When reading palette data only, read the current data pointed to
        // by self.vram_address, not what was previously pointed to.
        self.registers.borrow_mut().ppu_data =
            if self.vram_address >= PALETTE_TABLE_START {
                memory.ppu_read(self.vram_address)
            } else {
                self.vram_data
            };

        self.clock.tick(self.rendering_enabled());
    }

    fn process_latch(&mut self, memory: &mut Memory, latch: DataLatch) {
        let value = latch.value;

        use RegisterType::*;
        use AccessMode::*;
        match (latch.register_type, latch.access_mode) {
            (OamData, Read) => {},
            (Mask | Status | OamAddr, Write) => {},

            (Ctrl, Write) => {
                if !self.nmi_was_enabled_last_cycle {
                    // Attempt to trigger the second (or higher) NMI of this frame.
                    if self.can_generate_nmi() {
                        self.should_generate_nmi = true;
                    }
                }

                self.nmi_was_enabled_last_cycle = self.registers.borrow().ctrl.nmi_enabled;
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
                    latch.register_type,
                ),
        }
    }

    fn render_background(&mut self, memory: &Memory, frame: &mut Frame) {
        let palette_table = memory.palette_table();
        frame.set_universal_background_rgb(palette_table.universal_background_rgb());

        let name_table_number = self.registers.borrow().ctrl.name_table_number;
        //let _name_table_mirroring = memory.name_table_mirroring();
        let background_table_side = self.registers.borrow().ctrl.background_table_side;
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

        let palette_table = memory.palette_table();
        let sprite_table_side = self.registers.borrow().ctrl.sprite_table_side;
        // FIXME: No more sprites will be found once the end of OAM is reached,
        // effectively hiding any sprites before OAM[OAMADDR].
        let sprites = self.oam.sprites();
        // Lower index sprites are drawn on top of higher index sprites.
        for i in (0..sprites.len()).rev() {
            let sprite = sprites[i];
            let is_sprite_0 = i == 0;
            let column = sprite.x_coordinate();
            let row = sprite.y_coordinate();
            let palette_table_index = sprite.palette_table_index();

            for row_in_sprite in 0..8 {
                let row =
                    if sprite.flip_vertically() {
                        row + 7 - row_in_sprite
                    } else {
                        row + row_in_sprite
                    };

                if row >= 240 {
                    // FIXME: The part of vertically flipped sprites that is
                    // off the screen should still be rendered.
                    break;
                }

                memory.pattern_table(sprite_table_side).render_sprite_sliver(
                    sprite,
                    is_sprite_0,
                    palette_table.sprite_palette(palette_table_index),
                    frame,
                    column,
                    row,
                    row_in_sprite as usize,
                );
            }
        }
    }

    fn rendering_enabled(&self) -> bool {
        self.registers.borrow().mask.sprites_enabled || self.registers.borrow().mask.background_enabled
    }

    fn can_generate_nmi(&self) -> bool {
        self.registers.borrow().status.vblank_active && self.registers.borrow().ctrl.nmi_enabled
    }

    fn write_oam(&mut self, value: u8) {
        let oam_addr = self.registers.borrow().oam_addr;
        self.oam[oam_addr] = value;
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

        let increment = self.registers.borrow().ctrl.vram_address_increment as u16;
        self.vram_address = self.vram_address.advance(increment);
    }

    fn write_vram(&mut self, memory: &mut Memory, value: u8) {
        memory.ppu_write(self.vram_address, value);
        let increment = self.registers.borrow().ctrl.vram_address_increment as u16;
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
