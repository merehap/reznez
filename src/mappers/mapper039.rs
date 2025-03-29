use crate::mapper::*;
use crate::mappers::mapper241::Mapper241;

// Identical to mapper 241?
pub struct Mapper039 {
    mapper241: Mapper241,
}

impl Mapper for Mapper039 {
    fn write_to_cartridge_space(&mut self, params: &mut MapperParams, cpu_address: u16, value: u8) {
        self.mapper241.write_to_cartridge_space(params, cpu_address, value);
    }

    fn layout(&self) -> Layout {
        self.mapper241.layout()
    }
}

impl Mapper039 {
    pub fn new() -> Self {
        Self { mapper241: Mapper241 }
    }
}
