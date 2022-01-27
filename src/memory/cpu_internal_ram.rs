use crate::memory::cpu_address::CpuAddress;
use crate::cpu::port_access::{PortAccess, AccessMode};
use crate::memory::stack::Stack;

pub const NMI_VECTOR: CpuAddress = CpuAddress::new(0xFFFA);
pub const RESET_VECTOR: CpuAddress = CpuAddress::new(0xFFFC);
pub const IRQ_VECTOR: CpuAddress = CpuAddress::new(0xFFFE);

// FIXME: Ports should be configurable, not hard-coded here,
// but I can't find any data structure that is efficient enough.
pub const PPUCTRL:   CpuAddress = CpuAddress::new(0x2000);
pub const PPUMASK:   CpuAddress = CpuAddress::new(0x2001);
pub const PPUSTATUS: CpuAddress = CpuAddress::new(0x2002);
pub const OAMADDR:   CpuAddress = CpuAddress::new(0x2003);
pub const OAMDATA:   CpuAddress = CpuAddress::new(0x2004);
pub const PPUSCROLL: CpuAddress = CpuAddress::new(0x2005);
pub const PPUADDR:   CpuAddress = CpuAddress::new(0x2006);
pub const PPUDATA:   CpuAddress = CpuAddress::new(0x2007);
pub const OAM_DMA:   CpuAddress = CpuAddress::new(0x4014);

pub const JOYSTICK_1_PORT: CpuAddress = CpuAddress::new(0x4016);
pub const JOYSTICK_2_PORT: CpuAddress = CpuAddress::new(0x4017);

pub struct CpuInternalRam {
    pub stack_pointer: u8,
    memory: Box<[u8; 0x10000]>,
    latch: Option<PortAccess>,
}

impl CpuInternalRam {
    pub fn new() -> CpuInternalRam {
        CpuInternalRam {
            stack_pointer: 0xFD,
            memory: Box::new([0; 0x10000]),
            latch: None,
        }
    }

    #[inline]
    pub fn read(&mut self, address: CpuAddress) -> u8 {
        let raw_address = address.to_raw() as usize;
        let value = self.memory[raw_address];
        if address == PPUSTATUS ||
            address == OAMDATA ||
            address == PPUDATA ||
            address == JOYSTICK_1_PORT ||
            address == JOYSTICK_2_PORT {

            self.latch = Some(PortAccess {
                address,
                value,
                access_mode: AccessMode::Read,
            });
        }

        value
    }

    #[inline]
    pub fn write(&mut self, address: CpuAddress, value: u8) {
        let raw_address = address.to_raw() as usize;
        if address == PPUCTRL ||
            address == PPUMASK ||
            address == PPUSTATUS ||
            address == OAMADDR ||
            address == OAMDATA ||
            address == PPUSCROLL ||
            address == PPUADDR ||
            address == PPUDATA ||
            address == OAM_DMA ||
            address == JOYSTICK_1_PORT {

            self.latch = Some(PortAccess {
                address,
                value,
                access_mode: AccessMode::Write,
            });
        }

        self.memory[raw_address] = value;
    }

    #[inline]
    pub fn bus_access_read(&self, address: CpuAddress) -> u8 {
        self.memory[address.to_raw() as usize]
    }

    #[inline]
    pub fn bus_access_write(&mut self, address: CpuAddress, value: u8) {
        self.memory[address.to_raw() as usize] = value;
    }

    pub fn latch(&self) -> Option<PortAccess> {
        self.latch
    }

    pub fn reset_latch(&mut self) {
        self.latch = None;
    }

    pub fn stack(&mut self) -> Stack {
        Stack::new((&mut self.memory[0x100..0x200]).try_into().unwrap(), &mut self.stack_pointer)
    }

    pub fn to_slice(&mut self) -> &mut [u8; 0x10000] {
        self.memory.as_mut()
    }
}
