use crate::ppu::address::Address;
use crate::ppu::clock::Clock;
use crate::ppu;
use crate::ppu::memory::Memory;
use crate::ppu::oam::Oam;
use crate::ppu::register::ctrl::Ctrl;
use crate::ppu::register::mask::Mask;
use crate::ppu::register::status::Status;
use crate::ppu::render::frame::Frame;

const FIRST_VBLANK_CYCLE: u64 = 3 * 27384;
const SECOND_VBLANK_CYCLE: u64 = 3 * 57165;

pub struct Ppu {
    memory: Memory,
    oam: Oam,
    status: Status,

    clock: Clock,

    address_latch: Option<u8>,
    vram_address: Address,
    vram_data: u8,

    x_scroll_offset: u8,
    y_scroll_offset: u8,

    vblank_just_started: bool,
    suppress_vblank_active: bool,
}

impl Ppu {
    pub fn new(memory: Memory) -> Ppu {
        Ppu {
            memory,
            oam: Oam::new(),
            status: Status::new(),

            clock: Clock::new(),

            address_latch: None,

            vram_address: Address::from_u16(0),
            vram_data: 0,

            x_scroll_offset: 0,
            y_scroll_offset: 0,

            vblank_just_started: false,
            suppress_vblank_active: false,
        }
    }

    pub fn status(&self) -> Status {
        self.status
    }

    pub fn clock(&self) -> &Clock {
        &self.clock
    }

    pub fn read_oam(&mut self, oam_address: u8) -> u8 {
        self.oam[oam_address]
    }

    pub fn write_oam(&mut self, oam_address: u8, value: u8) {
        self.oam[oam_address] = value;
    }

    pub fn vram_data(&self) -> u8 {
        // When reading palette data only, read the current data pointed to
        // by self.vram_address, not what was previously pointed to.
        if self.vram_address >= ppu::memory::PALETTE_TABLE_START {
            self.memory.read(self.vram_address)
        } else {
            self.vram_data
        }
    }

    pub fn update_vram_data(&mut self, ctrl: Ctrl) {
        self.vram_data = self.memory.read(self.vram_address);
        let increment = ctrl.vram_address_increment as u8;
        self.vram_address = self.vram_address.advance(increment);
    }

    pub fn write_vram(&mut self, ctrl: Ctrl, value: u8) {
        self.memory.write(self.vram_address, value);
        let increment = ctrl.vram_address_increment as u8;
        self.vram_address = self.vram_address.advance(increment);
    }

    pub fn write_partial_vram_address(&mut self, value: u8) {
        if let Some(upper) = self.address_latch {
            self.vram_address = Address::from_u16((u16::from(upper) << 8) + u16::from(value));
            self.address_latch = None;
        } else {
            self.address_latch = Some(value);
        }
    }

    pub fn write_scroll_dimension(&mut self, dimension: u8) {
        if let Some(x_scroll_offset) = self.address_latch {
            self.x_scroll_offset = x_scroll_offset;
            self.y_scroll_offset = dimension;
            self.address_latch = None;
        } else {
            self.address_latch = Some(dimension);
        }
    }

    pub fn reset_address_latch(&mut self) {
        self.address_latch = None;
    }

    pub fn can_generate_nmi(&self, ctrl: Ctrl) -> bool {
        self.status.vblank_active && ctrl.nmi_enabled
    }

    pub fn vblank_just_started(&self) -> bool {
        self.vblank_just_started
    }

    pub fn stop_vblank(&mut self) {
        self.status.vblank_active = false;
        // https://wiki.nesdev.org/w/index.php?title=NMI#Race_condition
        if self.clock.scanline() == 241 && self.clock.cycle() == 1 {
            self.suppress_vblank_active = true;
        }
    }

    pub fn step(&mut self, ctrl: Ctrl, mask: Mask, frame: &mut Frame) {
        let total_cycles = self.clock().total_cycles();
        self.vblank_just_started = false;

        // TODO: Fix the first and second vblank cycles to not be special-cased if possible.
        if total_cycles == FIRST_VBLANK_CYCLE || total_cycles == SECOND_VBLANK_CYCLE {
            // TODO: Why don't we have the following enabled here?
            // Maybe just need to have "= false" to end it too.
            // self.status.vblank_active = true;
            self.vblank_just_started = true;
        } else if total_cycles < SECOND_VBLANK_CYCLE {
            // Do nothing.
        } else if self.clock.scanline() == 241 && self.clock.cycle() == 1 {
            if !self.suppress_vblank_active {
                self.status.vblank_active = true;
            }

            self.suppress_vblank_active = false;
            self.vblank_just_started = true;
        } else if self.clock.scanline() == 261 && self.clock.cycle() == 1 {
            self.status.vblank_active = false;
            self.status.sprite0_hit = false;
        } else if self.clock.scanline() == 1 && self.clock.cycle() == 1 {
            if mask.background_enabled {
                self.render_background(ctrl, frame);
            }

            if mask.sprites_enabled {
                self.render_sprites(ctrl, frame);
            }
        }

        let sprite_0 = self.oam.sprite_0();
        // TODO: Sprite 0 hit needs lots more work.
        if self.clock.scanline() == sprite_0.y_coordinate() as u16 &&
            self.clock.cycle() == 340 &&
            self.clock.cycle() > sprite_0.x_coordinate() as u16 &&
            mask.sprites_enabled && mask.background_enabled {

            self.status.sprite0_hit = true;
        }

        self.clock.tick(self.rendering_enabled(mask));
    }

    fn render_background(&mut self, ctrl: Ctrl, frame: &mut Frame) {
        let palette_table = self.memory.palette_table();
        frame.set_universal_background_rgb(palette_table.universal_background_rgb());

        let name_table_number = ctrl.name_table_number;
        let _name_table_mirroring = self.memory.name_table_mirroring();
        let background_table_side = ctrl.background_table_side;
        self.memory.name_table(name_table_number).render(
            &self.memory.pattern_table(),
            background_table_side,
            &palette_table,
            -(self.x_scroll_offset as i16),
            -(self.y_scroll_offset as i16),
            frame,
        );
        self.memory.name_table(name_table_number.next_horizontal()).render(
            &self.memory.pattern_table(),
            background_table_side,
            &palette_table,
            -(self.x_scroll_offset as i16) + 256,
            -(self.y_scroll_offset as i16),
            frame,
        );
    }

    fn render_sprites(&mut self, ctrl: Ctrl, frame: &mut Frame) {
        frame.clear_sprite_buffer();

        let palette_table = self.memory.palette_table();
        let sprite_table_side = ctrl.sprite_table_side;
        let sprites = self.oam.sprites();
        // Lower index sprites are drawn on top of higher index sprites.
        for i in (0..sprites.len()).rev() {
            let sprite = sprites[i];
            let is_sprite_0 = i == 0;
            let column = sprite.x_coordinate();
            let row = sprite.y_coordinate();
            let palette_table_index = sprite.palette_table_index();

            for row_in_sprite in 0..8 {
                if row + row_in_sprite >= 240 {
                    break;
                }

                let row =
                    if sprite.flip_vertically() {
                        row + 7 - row_in_sprite
                    } else {
                        row + row_in_sprite
                    };

                self.memory.pattern_table().render_sprite_sliver(
                    sprite_table_side,
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

    fn rendering_enabled(&self, mask: Mask) -> bool {
        mask.sprites_enabled || mask.background_enabled
    }
}
