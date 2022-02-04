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

    pub(super) latch: Option<DataLatch>,
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

            latch: None,
        }
    }

    pub fn read(&mut self, register_type: RegisterType) -> u8 {
        use RegisterType::*;
        let value =
            match register_type {
                Ctrl => {/* TODO: Open bus behavior here. */ 0},
                Mask => {/* TODO: Open bus behavior here. */ 0},
                Status => {
                    // TODO: Open bus behavior here for the unused bits.
                    self.status.to_u8()
                },
                OamAddr => {/* TODO: Open bus behavior here. */ 0},
                OamData => self.oam_data,
                Scroll => {/* TODO: Open bus behavior here. */ 0},
                PpuAddr => {/* TODO: Open bus behavior here. */ 0},
                PpuData => self.ppu_data,
            };

        self.latch = Some(
            DataLatch {
                register_type,
                value,
                access_mode: AccessMode::Read,
            }
        );

        value
    }

    pub fn write(&mut self, register_type: RegisterType, value: u8) {
        self.latch = Some(
            DataLatch {
                register_type,
                value,
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

    pub fn latch(&self) -> Option<DataLatch> {
        self.latch
    }

    pub fn reset_latch(&mut self) {
        self.latch = None;
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
pub struct DataLatch {
    pub(super) register_type: RegisterType,
    pub(super) value: u8,
    pub(super) access_mode: AccessMode,
}

#[derive(Clone, Copy)]
pub enum AccessMode {
    Read,
    Write,
}
