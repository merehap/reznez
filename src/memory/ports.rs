use phf::phf_set;

use crate::memory::cpu_address::CpuAddress;
use crate::memory::port_access::{PortAccess, AccessMode};

static PORT_ADDRESSES: phf::Set<u16> =
    phf_set! {
        // PPUCTRL
        0x2000u16,
        // PPUMASK
        0x2001u16,
        // PPUSTATUS
        0x2002u16,
        // OAMADDR
        0x2003u16,
        // OAMDATA
        0x2004u16,
        // PPUSCROLL
        0x2005u16,
        // PPUADDR
        0x2006u16,
        // PPUDATA
        0x2007u16,
        // OAM_DMA
        0x4014u16,
        // JOYSTICK_1_PORT
        0x4016u16,
        // JOYSTICK_2_PORT
        0x4017u16,
    };

static READ_LATCH_PORT_ADDRESSES: phf::Set<u16> =
    phf_set! {
        // PPUSTATUS
        0x2002u16,
        // OAMDATA
        0x2004u16,
        // PPUDATA
        0x2007u16,
        // JOYSTICK_1_PORT
        0x4016u16,
        // JOYSTICK_2_PORT
        0x4017u16,
    };

// Only JOYSTICK_2_PORT is missing.
static WRITE_LATCH_PORT_ADDRESSES: phf::Set<u16> =
    phf_set! {
        // PPUCTRL
        0x2000u16,
        // PPUMASK
        0x2001u16,
        // PPUSTATUS
        0x2002u16,
        // OAMADDR
        0x2003u16,
        // OAMDATA
        0x2004u16,
        // PPUSCROLL
        0x2005u16,
        // PPUADDR
        0x2006u16,
        // PPUDATA
        0x2007u16,
        // OAM_DMA
        0x4014u16,
        // JOYSTICK_1_PORT
        0x4016u16,
    };

pub struct Ports {
    ports: [u8; 0x10000],
    latch: Option<PortAccess>,
}

impl Ports {
    pub fn new() -> Ports {
        Ports {
            ports: [0; 0x10000],
            latch: None,
        }
    }

    #[inline]
    pub fn get(&mut self, address: CpuAddress) -> u8 {
        assert!(PORT_ADDRESSES.contains(&address.to_raw()));

        // FIXME: Should return the latch's value if this is a write-only port.
        let value = self.ports[address.to_usize()];
        if READ_LATCH_PORT_ADDRESSES.contains(&address.to_raw()) {
            self.latch = Some(PortAccess {
                address,
                value,
                access_mode: AccessMode::Read,
            });
        }

        value
    }

    #[inline]
    pub fn set(&mut self, address: CpuAddress, value: u8) {
        assert!(PORT_ADDRESSES.contains(&address.to_raw()));

        if WRITE_LATCH_PORT_ADDRESSES.contains(&address.to_raw()) {
            self.latch = Some(PortAccess {
                address,
                value,
                access_mode: AccessMode::Write,
            });
        }

        // FIXME: What do we return for JOYPAD_2?
        self.ports[address.to_usize()] = value;
    }

    #[inline]
    pub fn bus_access_read(&self, address: CpuAddress) -> u8 {
        self.ports[address.to_usize()]
    }

    #[inline]
    pub fn bus_access_write(&mut self, address: CpuAddress, value: u8) {
        self.ports[address.to_usize()] = value;
    }

    pub fn latch(&self) -> Option<PortAccess> {
        self.latch
    }

    pub fn reset_latch(&mut self) {
        self.latch = None;
    }
}
