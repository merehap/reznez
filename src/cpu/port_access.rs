use crate::cpu::address::Address;

#[derive(Clone, Copy)]
pub struct PortAccess {
    pub address: Address,
    pub value: u8,
    pub access_mode: AccessMode,
}

#[derive(Clone, Copy)]
pub enum AccessMode {
    Read,
    Write,
}
