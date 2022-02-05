use num_derive::FromPrimitive;

use crate::ppu::registers::ctrl;
use crate::ppu::registers::ctrl::Ctrl;
use crate::ppu::registers::mask;
use crate::ppu::registers::mask::Mask;
use crate::ppu::registers::status::Status;

// Manually set to be equal to the number of RegisterTypes.
pub struct PpuRegisters {
    pub(super) ctrl: Ctrl,
    pub(super) mask: Mask,
    pub(super) status: Status,
    pub(super) oam_addr: u8,
    pub(super) oam_data: u8,
    pub(super) scroll: u8,
    pub(super) ppu_addr: u8,
    pub(super) ppu_data: u8,

    pub(super) latch: u8,
    pub(super) latch_access: Option<LatchAccess>,
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
            ppu_data: 0,

            latch: 0,
            latch_access: None,
        }
    }

    pub fn read(&mut self, register_type: RegisterType) -> u8 {
        use RegisterType::*;
        let value =
            match register_type {
                Ctrl => None,
                Mask => None,
                Status => {
                    // TODO: Open bus behavior here for the unused bits.
                    Some(self.status.to_u8())
                },
                OamAddr => None,
                OamData => Some(self.oam_data),
                Scroll => None,
                PpuAddr => None,
                PpuData => Some(self.ppu_data),
            };

        if let Some(value) = value {
            self.latch = value;
            self.latch_access = Some(
                LatchAccess {
                    register_type,
                    access_mode: AccessMode::Read,
                }
            );
        }

        /* TODO: Open bus behavior here. */
        value.unwrap_or(0)
    }

    pub fn write(&mut self, register_type: RegisterType, value: u8) {
        self.latch = value;
        self.latch_access = Some(
            LatchAccess {
                register_type,
                access_mode: AccessMode::Write,
            }
        );

        use RegisterType::*;
        match register_type {
            Ctrl => self.ctrl = ctrl::Ctrl::from_u8(value),
            Mask => self.mask = mask::Mask::from_u8(value),
            Status => {/* TODO: Open bus behavior here. */},
            OamAddr => self.oam_addr = value,
            OamData => self.oam_data = value,
            Scroll => self.scroll = value,
            PpuAddr => self.ppu_addr = value,
            PpuData => self.ppu_data = value,
        }
    }
}

#[derive(Clone, Copy, Debug, FromPrimitive)]
pub enum RegisterType {
    Ctrl,
    Mask,
    Status,
    OamAddr,
    OamData,
    Scroll,
    PpuAddr,
    PpuData,
}

#[derive(Clone, Copy)]
pub struct LatchAccess {
    pub(super) register_type: RegisterType,
    pub(super) access_mode: AccessMode,
}

#[derive(Clone, Copy)]
pub enum AccessMode {
    Read,
    Write,
}
