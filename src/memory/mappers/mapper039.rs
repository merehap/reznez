use crate::memory::mapper::*;
use crate::memory::mappers::mapper241::Mapper241;

// Identical to mapper 241?
pub struct Mapper039 {
    mapper241: Mapper241,
}

impl Mapper for Mapper039 {
    fn initial_layout(&self) -> InitialLayout {
        self.mapper241.initial_layout()
    }

    fn write_to_cartridge_space(&mut self, params: &mut MapperParams, cpu_address: CpuAddress, value: u8) {
        self.mapper241.write_to_cartridge_space(params, cpu_address, value);
    }
}

impl Mapper039 {
    pub fn new() -> Self {
        Self { mapper241: Mapper241 }
    }
}
