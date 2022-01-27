use crate::memory::cpu_address::CpuAddress;

#[derive(Clone, Copy)]
pub struct PortAccess {
    pub address: CpuAddress,
    pub value: u8,
    pub access_mode: AccessMode,
}

#[derive(Clone, Copy)]
pub enum AccessMode {
    Read,
    Write,
}
