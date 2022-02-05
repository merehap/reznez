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

    latch: u8,
    latch_access: Option<LatchAccess>,

    frames_until_decay: Option<u8>,
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

            frames_until_decay: None,
        }
    }

    pub(super) fn latch(&self) -> u8 {
        self.latch
    }

    pub(super) fn consume_latch_access(&mut self) -> Option<LatchAccess> {
        let result = self.latch_access;
        self.latch_access = None;
        result
    }

    pub fn read(&mut self, register_type: RegisterType) -> u8 {
        use RegisterType::*;
        let value =
            match register_type {
                // Write-only registers.
                Ctrl | Mask | OamAddr | Scroll | PpuAddr => None,
                // Retain the open bus values for the unused bits of Status.
                Status => Some(self.status.to_u8() | (self.latch & 0b0001_1111)),
                OamData => Some(self.oam_data),
                PpuData => Some(self.ppu_data),
            };

        // If a readable register is read from, update the latch.
        if let Some(value) = value {
            self.latch = value;
            self.latch_access = Some(
                LatchAccess {
                    register_type,
                    access_mode: AccessMode::Read,
                }
            );

            // At least one frame should occur before the latch decays to zero.
            self.frames_until_decay = Some(2);
        }

        // Reads to write-only registers return the latch (open bus behavior).
        value.unwrap_or(self.latch)
    }

    pub fn write(&mut self, register_type: RegisterType, value: u8) {
        self.latch = value;
        self.latch_access = Some(
            LatchAccess {
                register_type,
                access_mode: AccessMode::Write,
            }
        );

        // At least one frame should occur before the latch decays to zero.
        self.frames_until_decay = Some(2);

        use RegisterType::*;
        match register_type {
            Ctrl => self.ctrl = ctrl::Ctrl::from_u8(value),
            Mask => self.mask = mask::Mask::from_u8(value),
            Status => {/* Read-only. */},
            OamAddr => self.oam_addr = value,
            OamData => self.oam_data = value,
            Scroll => self.scroll = value,
            PpuAddr => self.ppu_addr = value,
            PpuData => self.ppu_data = value,
        }
    }

    pub fn maybe_decay_latch(&mut self) {
        match self.frames_until_decay {
            None => {/* The latch has already decayed. */},
            Some(0) => {
                self.latch = 0b0000_0000;
                self.frames_until_decay = None;
            },
            Some(frames) => self.frames_until_decay = Some(frames - 1),
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
