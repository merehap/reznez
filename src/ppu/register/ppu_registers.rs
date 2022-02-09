use crate::ppu::name_table::name_table_number::NameTableNumber;
use crate::ppu::pattern_table::PatternTableSide;
use crate::ppu::register::ppu_register_latch::PpuRegisterLatch;
use crate::ppu::register::register_type::RegisterType;
use crate::ppu::register::registers::ctrl;
use crate::ppu::register::registers::ctrl::{Ctrl, SpriteHeight, VramAddressIncrement};
use crate::ppu::register::registers::mask;
use crate::ppu::register::registers::mask::Mask;
use crate::ppu::register::registers::ppu_data;
use crate::ppu::register::registers::ppu_data::PpuData;
use crate::ppu::register::registers::status::Status;

pub struct PpuRegisters {
    pub(in crate::ppu) ctrl: Ctrl,
    pub(in crate::ppu) mask: Mask,
    pub(in crate::ppu) status: Status,
    pub(in crate::ppu) oam_addr: u8,
    pub(in crate::ppu) oam_data: u8,
    pub(in crate::ppu) scroll: u8,
    pub(in crate::ppu) ppu_addr: u8,
    pub(in crate::ppu) ppu_data: PpuData,

    latch: PpuRegisterLatch,
    latch_access: Option<LatchAccess>,
}

impl PpuRegisters {
    pub fn new() -> PpuRegisters {
        PpuRegisters {
            ctrl: Ctrl::new(),
            mask: Mask::new(),
            status: Status::new(),
            oam_addr: 0,
            oam_data: 0,
            scroll: 0,
            ppu_addr: 0,
            ppu_data: PpuData {value: 0, is_palette_data: false},

            latch: PpuRegisterLatch::new(),
            latch_access: None,
        }
    }

    pub(in crate::ppu) fn nmi_enabled(&self) -> bool {
        self.ctrl.nmi_enabled
    }

    pub(in crate::ppu) fn sprite_height(&self) -> SpriteHeight {
        self.ctrl.sprite_height
    }

    pub(in crate::ppu) fn name_table_number(&self) -> NameTableNumber {
        self.ctrl.name_table_number
    }

    pub(in crate::ppu) fn background_table_side(&self) -> PatternTableSide {
        self.ctrl.background_table_side
    }

    pub(in crate::ppu) fn sprite_table_side(&self) -> PatternTableSide {
        self.ctrl.sprite_table_side
    }

    pub(in crate::ppu) fn vram_address_increment(&self) -> VramAddressIncrement {
        self.ctrl.vram_address_increment
    }

    pub(in crate::ppu) fn background_enabled(&self) -> bool {
        self.mask.background_enabled
    }

    pub(in crate::ppu) fn sprites_enabled(&self) -> bool {
        self.mask.sprites_enabled
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
        let register_value =
            match register_type {
                // Write-only registers.
                Ctrl | Mask | OamAddr | Scroll | PpuAddr => None,
                // Retain the open bus values for the unused bits of Status.
                Status => Some(self.status.to_u8() | (self.latch.value() & 0b0001_1111)),
                OamData => Some(self.oam_data),
                PpuData if self.ppu_data.is_palette_data =>
                    Some(self.ppu_data.value | (self.latch.value() & 0b1100_0000)),
                PpuData => Some(self.ppu_data.value),
            };

        // If a readable register is read from, update the latch.
        if let Some(register_value) = register_value {
            self.latch_access = Some(
                LatchAccess {
                    register_type,
                    access_mode: AccessMode::Read,
                }
            );

            self.latch.update_from_read(register_type, register_value);
        }

        self.latch.value()
    }

    pub fn write(&mut self, register_type: RegisterType, register_value: u8) {
        self.latch_access = Some(
            LatchAccess {
                register_type,
                access_mode: AccessMode::Write,
            }
        );

        self.latch.update_from_write(register_value);

        use RegisterType::*;
        match register_type {
            Ctrl => self.ctrl = ctrl::Ctrl::from_u8(register_value),
            Mask => self.mask = mask::Mask::from_u8(register_value),
            Status => {/* Read-only. */},
            OamAddr => self.oam_addr = register_value,
            OamData => self.oam_data = register_value,
            Scroll => self.scroll = register_value,
            PpuAddr => self.ppu_addr = register_value,
            PpuData => self.ppu_data =
                ppu_data::PpuData {
                    value: register_value,
                    is_palette_data: false,
                },
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
