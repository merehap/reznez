use ux::u11;

use crate::mapper::ReadResult;

const RAM_SIZE: usize = 0x2000;

pub struct CpuInternalRam(Box<[u8; RAM_SIZE]>);

impl CpuInternalRam {
    pub fn new() -> CpuInternalRam {
        CpuInternalRam(Box::new([0; RAM_SIZE]))
    }

    pub fn peek(&self, index: u11) -> ReadResult {
        ReadResult::full(self.0[usize::from(u16::from(index))])
    }

    pub fn write(&mut self, index: u11, value: u8) {
        self.0[usize::from(u16::from(index))] = value;
    }
}