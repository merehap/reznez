use log::info;

use crate::memory::ppu::ppu_address::{PpuAddress, XScroll, YScroll};
use crate::ppu::clock::Clock;
use crate::ppu::name_table::name_table_quadrant::NameTableQuadrant;
use crate::ppu::pattern_table::PatternTableSide;
use crate::ppu::register::ppu_io_bus::PpuIoBus;
use crate::ppu::register::register_type::RegisterType;
use crate::ppu::register::registers::ctrl;
use crate::ppu::register::registers::ctrl::{AddressIncrement, Ctrl};
use crate::ppu::register::registers::mask::Mask;
use crate::ppu::register::registers::status::Status;
use crate::ppu::sprite::oam_address::OamAddress;
use crate::ppu::sprite::sprite_height::SpriteHeight;

#[derive(Clone)]
pub struct PpuRegisters {
    pub(in crate::ppu) ctrl: Ctrl,
    mask: Mask,
    pub(in crate::ppu) status: Status,
    pub oam_addr: OamAddress,
    pub(in crate::ppu) pending_ppu_data: u8,

    pub(in crate::ppu) current_address: PpuAddress,
    pub(in crate::ppu) next_address: PpuAddress,

    pub ppu_io_bus: PpuIoBus,
    io_bus_access: Option<LatchAccess>,
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
            io_bus_access: None,
        }
    }

    pub fn nmi_enabled(&self) -> bool {
        self.ctrl.nmi_enabled
    }

    pub fn sprite_height(&self) -> SpriteHeight {
        self.ctrl.sprite_height
    }

    pub fn background_table_side(&self) -> PatternTableSide {
        self.ctrl.background_table_side
    }

    pub fn sprite_table_side(&self) -> PatternTableSide {
        self.ctrl.sprite_table_side
    }

    pub(in crate::ppu) fn current_address_increment(&self) -> AddressIncrement {
        self.ctrl.current_address_increment
    }

    pub fn base_name_table_quadrant(&self) -> NameTableQuadrant {
        self.ctrl.base_name_table_quadrant
    }

    pub fn mask(&self) -> Mask {
        self.mask
    }

    pub fn background_enabled(&self) -> bool {
        self.mask.background_enabled
    }

    pub fn sprites_enabled(&self) -> bool {
        self.mask.sprites_enabled
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

    pub(in crate::ppu) fn ppu_io_bus_value(&self) -> u8 {
        self.ppu_io_bus.value()
    }

    pub(in crate::ppu) fn maybe_decay_ppu_io_bus(&mut self) {
        self.ppu_io_bus.maybe_decay();
    }

    pub(in crate::ppu) fn take_io_bus_access(&mut self) -> Option<LatchAccess> {
        self.io_bus_access.take()
    }

    pub fn rendering_enabled(&self) -> bool {
        self.mask.sprites_enabled || self.mask.background_enabled
    }

    pub fn can_generate_nmi(&self) -> bool {
        self.status.vblank_active && self.ctrl.nmi_enabled
    }

    pub fn peek(
        &self,
        register_type: RegisterType,
        peek: impl FnMut(PpuAddress) -> u8
    ) -> u8 {
        self.check(register_type, peek)
            .unwrap_or(self.ppu_io_bus.value())
    }

    pub fn read(
        &mut self,
        register_type: RegisterType,
        mut read: impl FnMut(PpuAddress) -> u8,
    ) -> u8 {
        // If a readable register is read from, update the ppu_io_bus.
        if let Some(register_value) = self.check(register_type, &mut read) {
            self.io_bus_access =
                Some(LatchAccess { register_type, access_mode: AccessMode::Read });

            self.ppu_io_bus.update_from_read(register_type, register_value);
        }

        if register_type == RegisterType::PpuData {
            self.pending_ppu_data = read(self.current_address.to_pending_data_source());
            self.current_address.advance(self.current_address_increment());
        }

        self.ppu_io_bus.value()
    }

    pub fn write(&mut self, register_type: RegisterType, register_value: u8) {
        self.io_bus_access =
            Some(LatchAccess { register_type, access_mode: AccessMode::Write });

        self.ppu_io_bus.update_from_write(register_value);

        use RegisterType::*;
        match register_type {
            Ctrl => self.ctrl = ctrl::Ctrl::from_u8(register_value),
            Mask => self.mask.set(register_value),
            Status => { /* Read-only. */ }
            OamAddr => self.oam_addr = OamAddress::from_u8(register_value),
            OamData => {}
            Scroll => {}
            PpuAddr => {}
            PpuData => self.current_address.advance(self.current_address_increment()),
        }
    }

    fn check(
        &self,
        register_type: RegisterType,
        mut access: impl FnMut(PpuAddress) -> u8,
    ) -> Option<u8> {
        use RegisterType::*;
        match register_type {
            // Write-only registers.
            Ctrl | Mask | OamAddr | OamData | Scroll | PpuAddr => None,
            // Retain the previous ppu_io_bus values for the unused bits of Status.
            Status => Some(self.status.to_u8() | (self.ppu_io_bus.value() & 0b0001_1111)),
            PpuData if self.current_address >= PpuAddress::PALETTE_TABLE_START => {
                // When reading palette data only, read the current data pointed to
                // by self.current_address, not what was previously pointed to.
                let value = access(self.current_address);
                // Retain the previous ppu_io_bus values for the unused bits of palette data.
                let high_ppu_io_bus = self.ppu_io_bus.value() & 0b1100_0000;
                Some(value | high_ppu_io_bus)
            }
            PpuData => Some(self.pending_ppu_data),
        }
    }
}

#[derive(Clone, Copy)]
pub struct LatchAccess {
    pub register_type: RegisterType,
    pub access_mode: AccessMode,
}

#[derive(Clone, Copy)]
pub enum AccessMode {
    Read,
    Write,
}
