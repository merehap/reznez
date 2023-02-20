use crate::memory::ppu::ppu_address::{PpuAddress, XScroll, YScroll};
use crate::ppu::name_table::name_table_quadrant::NameTableQuadrant;
use crate::ppu::pattern_table::PatternTableSide;
use crate::ppu::register::ppu_register_latch::PpuRegisterLatch;
use crate::ppu::register::register_type::RegisterType;
use crate::ppu::register::registers::ctrl;
use crate::ppu::register::registers::ctrl::{AddressIncrement, Ctrl};
use crate::ppu::register::registers::mask;
use crate::ppu::register::registers::mask::Mask;
use crate::ppu::register::registers::ppu_data::PpuData;
use crate::ppu::register::registers::status::Status;
use crate::ppu::sprite::sprite_height::SpriteHeight;

#[derive(Clone)]
pub struct PpuRegisters {
    pub(in crate::ppu) ctrl: Ctrl,
    pub mask: Mask,
    pub(in crate::ppu) status: Status,
    pub(in crate::ppu) oam_addr: u8,
    pub(in crate::ppu) oam_data: u8,
    pub(in crate::ppu) ppu_data: PpuData,
    pub(in crate::ppu) pending_ppu_data: u8,

    pub(in crate::ppu) current_address: PpuAddress,
    pub(in crate::ppu) next_address: PpuAddress,

    latch: PpuRegisterLatch,
    latch_access: Option<LatchAccess>,
}

impl PpuRegisters {
    pub fn new() -> PpuRegisters {
        PpuRegisters {
            ctrl: Ctrl::new(),
            mask: Mask::all_disabled(),
            status: Status::new(),
            oam_addr: 0,
            oam_data: 0,
            ppu_data: PpuData { value: 0, is_palette_data: false },
            pending_ppu_data: 0,

            current_address: PpuAddress::ZERO,
            next_address: PpuAddress::ZERO,

            latch: PpuRegisterLatch::new(),
            latch_access: None,
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

    pub(in crate::ppu) fn start_vblank(&mut self) {
        self.status.vblank_active = true;
    }

    pub(in crate::ppu) fn stop_vblank(&mut self) {
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

    pub(in crate::ppu) fn latch_value(&self) -> u8 {
        self.latch.value()
    }

    pub(in crate::ppu) fn maybe_decay_latch(&mut self) {
        self.latch.maybe_decay();
    }

    pub(in crate::ppu) fn take_latch_access(&mut self) -> Option<LatchAccess> {
        self.latch_access.take()
    }

    pub fn rendering_enabled(&self) -> bool {
        self.mask.sprites_enabled || self.mask.background_enabled
    }

    pub fn can_generate_nmi(&self) -> bool {
        self.status.vblank_active && self.ctrl.nmi_enabled
    }

    pub fn read(&mut self, register_type: RegisterType) -> u8 {
        use RegisterType::*;
        let register_value = match register_type {
            // Write-only registers.
            Ctrl | Mask | OamAddr | Scroll | PpuAddr => None,
            // Retain the previous latch values for the unused bits of Status.
            Status => Some(self.status.to_u8() | (self.latch.value() & 0b0001_1111)),
            OamData => Some(self.oam_data),
            // Retain the previous latch values for the unused bits of palette data.
            PpuData if self.ppu_data.is_palette_data => {
                Some(self.ppu_data.value | (self.latch.value() & 0b1100_0000))
            }
            PpuData => Some(self.ppu_data.value),
        };

        // If a readable register is read from, update the latch.
        if let Some(register_value) = register_value {
            self.latch_access =
                Some(LatchAccess { register_type, access_mode: AccessMode::Read });

            self.latch.update_from_read(register_type, register_value);
        }

        self.latch.value()
    }

    pub fn write(&mut self, register_type: RegisterType, register_value: u8) {
        self.latch_access =
            Some(LatchAccess { register_type, access_mode: AccessMode::Write });

        self.latch.update_from_write(register_value);

        use RegisterType::*;
        match register_type {
            Ctrl => self.ctrl = ctrl::Ctrl::from_u8(register_value),
            Mask => self.mask = mask::Mask::from_u8(register_value),
            Status => { /* Read-only. */ }
            OamAddr => self.oam_addr = register_value,
            OamData => self.oam_data = register_value,
            Scroll => {}
            PpuAddr => {}
            PpuData => { /* Writing to PpuData already stored the value to memory. */ }
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
