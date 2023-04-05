use crate::memory::mapper::*;
use crate::memory::mappers::mapper003::Mapper003;

// Clone of CNROM but without bus conflicts (which yet supported).
pub struct Mapper101 {
    mapper003: Mapper003,
}

impl Mapper for Mapper101 {
    fn initial_layout(&self) -> InitialLayout {
        self.mapper003.initial_layout()
    }

    fn write_to_cartridge_space(&mut self, params: &mut MapperParams, cpu_address: CpuAddress, value: u8) {
        self.mapper003.write_to_cartridge_space(params, cpu_address, value);
    }
}

impl Mapper101 {
    pub fn new() -> Self {
        Mapper101 { mapper003: Mapper003 }
    }
}
