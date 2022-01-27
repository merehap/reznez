use crate::memory::cpu_address::CpuAddress;
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
}

impl CpuInternalRam {
    pub fn new() -> CpuInternalRam {
        CpuInternalRam {
            stack_pointer: 0xFD,
            memory: Box::new([0; 0x10000]),
        }
    }

    #[inline]
    pub fn read(&self, address: CpuAddress) -> u8 {
        self.memory[address.to_usize()]
    }

    #[inline]
    pub fn write(&mut self, address: CpuAddress, value: u8) {
        self.memory[address.to_usize()] = value;
    }

    #[inline]
    pub fn bus_access_read(&self, address: CpuAddress) -> u8 {
        self.memory[address.to_raw() as usize]
    }

    #[inline]
    pub fn bus_access_write(&mut self, address: CpuAddress, value: u8) {
        self.memory[address.to_raw() as usize] = value;
    }

    pub fn stack(&mut self) -> Stack {
        Stack::new((&mut self.memory[0x100..0x200]).try_into().unwrap(), &mut self.stack_pointer)
    }
}
