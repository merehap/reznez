use log::info;

use crate::memory::ppu::ppu_address::{PpuAddress, XScroll, YScroll};
use crate::ppu::clock::Clock;
use crate::ppu::name_table::name_table_quadrant::NameTableQuadrant;
use crate::ppu::pattern_table::PatternTableSide;
use crate::ppu::register::ppu_io_bus::PpuIoBus;
use crate::ppu::register::registers::ctrl;
use crate::ppu::register::registers::ctrl::{AddressIncrement, Ctrl};
use crate::ppu::register::registers::mask::Mask;
use crate::ppu::register::registers::status::Status;
use crate::ppu::sprite::oam::Oam;
use crate::ppu::sprite::oam_address::OamAddress;
use crate::ppu::sprite::sprite_height::SpriteHeight;

#[derive(Clone)]
pub struct PpuRegisters {
    ctrl: Ctrl,
    mask: Mask,
    status: Status,
    pub oam_addr: OamAddress,
    pending_ppu_data: u8,

    pub(in crate::ppu) current_address: PpuAddress,
    pub(in crate::ppu) next_address: PpuAddress,

    ppu_io_bus: PpuIoBus,

    pub suppress_vblank_active: bool,
    pub nmi_requested: bool,
    nmi_was_enabled_last_cycle: bool,

    rendering_enabled: bool,
    write_toggle: WriteToggle,
    rendering_toggle_state: RenderingToggleState,
    status_read: bool,
}

impl PpuRegisters {
    pub fn new() -> PpuRegisters {
        PpuRegisters {
            ctrl: Ctrl::new(),
            mask: Mask::all_disabled(),
            status: Status::new(),
            oam_addr: OamAddress::new(),
            pending_ppu_data: 0,

            current_address: PpuAddress::ZERO,
            next_address: PpuAddress::ZERO,

            ppu_io_bus: PpuIoBus::new(),

            write_toggle: WriteToggle::FirstByte,
            nmi_requested: false,
            nmi_was_enabled_last_cycle: false,
            suppress_vblank_active: false,
            status_read: false,
            rendering_enabled: false,
            rendering_toggle_state: RenderingToggleState::Inactive,
        }
    }

    pub fn nmi_enabled(&self) -> bool {
        self.ctrl.nmi_enabled()
    }

    pub fn sprite_height(&self) -> SpriteHeight {
        self.ctrl.sprite_height()
    }

    pub fn background_table_side(&self) -> PatternTableSide {
        self.ctrl.background_table_side()
    }

    pub fn sprite_table_side(&self) -> PatternTableSide {
        self.ctrl.sprite_table_side()
    }

    pub(in crate::ppu) fn current_address_increment(&self) -> AddressIncrement {
        self.ctrl.current_address_increment()
    }

    pub fn base_name_table_quadrant(&self) -> NameTableQuadrant {
        self.ctrl.base_name_table_quadrant()
    }

    pub fn mask(&self) -> Mask {
        self.mask
    }

    pub fn background_enabled(&self) -> bool {
        self.mask.background_enabled()
    }

    pub fn sprites_enabled(&self) -> bool {
        self.mask.sprites_enabled()
    }

    pub fn rendering_enabled(&self) -> bool {
        self.rendering_enabled
    }

    pub fn pending_ppu_data(&self) -> u8 {
        self.pending_ppu_data
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

    pub fn write_toggle(&self) -> WriteToggle {
        self.write_toggle
    }

    pub(in crate::ppu) fn start_vblank(&mut self, clock: &Clock) {
        info!(target: "ppuflags", " {}\tStarting vblank.", clock);
        self.status.vblank_active = true;
    }

    pub(in crate::ppu) fn stop_vblank(&mut self, clock: &Clock) {
        if self.status.vblank_active {
            info!(target: "ppuflags", " {}\tStopping vblank.", clock);
        }

        self.status.vblank_active = false;
    }

    pub(in crate::ppu) fn set_sprite0_hit(&mut self) {
        self.status.sprite0_hit = true;
    }

    pub(in crate::ppu) fn clear_sprite0_hit(&mut self) {
        self.status.sprite0_hit = false;
    }

    pub(in crate::ppu) fn set_sprite_overflow(&mut self) {
        self.status.sprite_overflow = true;
    }

    pub(in crate::ppu) fn clear_sprite_overflow(&mut self) {
        self.status.sprite_overflow = false;
    }

    pub(in crate::ppu) fn maybe_decay_ppu_io_bus(&mut self, clock: &Clock) {
        if clock.cycle() == 1 {
            self.ppu_io_bus.maybe_decay();
        }
    }

    pub fn maybe_toggle_rendering_enabled(&mut self) {
        use RenderingToggleState::*;
        match self.rendering_toggle_state {
            Inactive => {}
            Pending => self.rendering_toggle_state = Ready,
            Ready => {
                self.rendering_enabled = !self.rendering_enabled;
                self.rendering_toggle_state = Inactive;
            }
        }
    }

    pub fn can_generate_nmi(&self) -> bool {
        self.status.vblank_active && self.ctrl.nmi_enabled()
    }

    pub fn peek_ppu_io_bus(&self) -> u8 {
        self.ppu_io_bus.value()
    }

    pub fn peek_status(&self) -> u8 {
        self.status.to_u8() | (self.ppu_io_bus.value() & 0b0001_1111)
    }

    // 0x2002
    pub fn read_status(&mut self) -> u8 {
        self.status_read = true;
        self.write_toggle = WriteToggle::FirstByte;

        let value = self.peek_status();
        self.ppu_io_bus.update_from_status_read(value);
        self.ppu_io_bus.value()
    }

    // TODO: This should be combined into read_status, but it can't currently access the clock.
    pub fn maybe_apply_status_read(&mut self, clock: &Clock) {
        if self.status_read {
            self.status_read = false;
            self.stop_vblank(clock);
            // https://wiki.nesdev.org/w/index.php?title=NMI#Race_condition
            if clock.scanline() == 241 && clock.cycle() == 1 {
                self.suppress_vblank_active = true;
            }
        }
    }

    pub fn peek_oam_data(&self, oam: &Oam) -> u8 {
        oam.peek(self.oam_addr)
    }

    pub fn read_oam_data(&mut self, oam: &Oam) -> u8 {
        let value = self.peek_oam_data(oam);
        self.ppu_io_bus.update_from_read(value);
        value
    }

    pub fn peek_ppu_data(&self, mut peeker: impl FnMut(PpuAddress) -> u8) -> u8 {
        if self.current_address >= PpuAddress::PALETTE_TABLE_START {
            // When reading palette data only, read the current data pointed to
            // by self.current_address, not what was previously pointed to.
            let value = peeker(self.current_address);
            // Retain the previous ppu_io_bus values for the unused bits of palette data.
            let high_ppu_io_bus = self.ppu_io_bus.value() & 0b1100_0000;
            value | high_ppu_io_bus
        } else {
            self.pending_ppu_data
        }
    }

    pub fn read_ppu_data(&mut self, mut reader: impl FnMut(PpuAddress) -> u8) -> u8 {
        let value = self.peek_ppu_data(&mut reader);
        self.ppu_io_bus.update_from_read(value);
        self.pending_ppu_data = reader(self.current_address.to_pending_data_source());
        self.current_address.advance(self.current_address_increment());
        self.ppu_io_bus.value()
    }

    pub fn write_ppu_io_bus(&mut self, register_value: u8) {
        self.ppu_io_bus.update_from_write(register_value);
    }

    // 0x2000
    pub fn write_ctrl(&mut self, value: u8) {
        self.ppu_io_bus.update_from_write(value);
        self.ctrl = ctrl::Ctrl::from_u8(value);

        self.next_address.set_name_table_quadrant(NameTableQuadrant::from_last_two_bits(value));
        // Potentially attempt to trigger the second (or higher) NMI of this frame.
        self.nmi_requested = !self.nmi_was_enabled_last_cycle;
        self.nmi_was_enabled_last_cycle = self.ctrl.nmi_enabled();
    }

    // 0x2001
    pub fn write_mask(&mut self, value: u8) {
        self.ppu_io_bus.update_from_write(value);
        self.mask.set(value);
        if self.rendering_enabled != (self.mask.sprites_enabled() || self.mask.background_enabled()) {
            self.rendering_toggle_state = RenderingToggleState::Pending;
        }
    }

    pub fn write_oam_addr(&mut self, value: u8) {
        self.ppu_io_bus.update_from_write(value);
        self.oam_addr = OamAddress::from_u8(value);
    }

    pub fn write_oam_data(&mut self, oam: &mut Oam, value: u8) {
        oam.write(self.oam_addr, value);
        self.ppu_io_bus.update_from_write(value);
        // Advance to next sprite byte to write.
        self.oam_addr.increment();
    }

    // 0x2005
    pub fn write_scroll(&mut self, dimension: u8) {
        match self.write_toggle {
            WriteToggle::FirstByte => self.next_address.set_x_scroll(dimension),
            WriteToggle::SecondByte => self.next_address.set_y_scroll(dimension),
        }

        self.write_toggle.toggle();
        self.ppu_io_bus.update_from_write(dimension);
    }

    // 0x2006
    pub fn write_ppu_addr(&mut self, value: u8) {
        match self.write_toggle {
            WriteToggle::FirstByte => self.next_address.set_high_byte(value),
            WriteToggle::SecondByte => {
                self.next_address.set_low_byte(value);
                self.current_address = self.next_address;
            }
        }

        self.write_toggle.toggle();
        self.write_ppu_io_bus(value);
    }

    pub fn write_ppu_data(&mut self, value: u8) {
        self.ppu_io_bus.update_from_write(value);
        self.current_address.advance(self.current_address_increment());
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum WriteToggle {
    FirstByte,
    SecondByte,
}

impl WriteToggle {
    pub fn toggle(&mut self) {
        use WriteToggle::*;
        *self = match self {
            FirstByte => SecondByte,
            SecondByte => FirstByte,
        };
    }
}

#[derive(Clone, Copy)]
pub enum RenderingToggleState {
    Inactive,
    Pending,
    Ready,
}
