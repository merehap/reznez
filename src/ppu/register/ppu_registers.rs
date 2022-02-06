use crate::ppu::register::ppu_register_latch::PpuRegisterLatch;
use crate::ppu::register::register_type::RegisterType;
use crate::ppu::register::registers::ctrl;
use crate::ppu::register::registers::ctrl::Ctrl;
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

    pub(in crate::ppu) fn latch_value(&self) -> u8 {
        self.latch.value()
    }

    pub(in crate::ppu) fn maybe_decay_latch(&mut self) {
        self.latch.maybe_decay();
    }

    pub(in crate::ppu) fn consume_latch_access(&mut self) -> Option<LatchAccess> {
        let result = self.latch_access;
        self.latch_access = None;
        result
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
