use crate::memory::mapper::*;
use crate::memory::mappers::mapper003::Mapper003;

// Clone of CNROM but without bus conflicts (which yet supported).
pub struct Mapper101 {
    mapper003: Mapper003,
}

impl Mapper for Mapper101 {
    fn write_to_cartridge_space(&mut self, cpu_address: CpuAddress, value: u8) {
        self.mapper003.write_to_cartridge_space(cpu_address, value);
    }

    fn params(&self) -> &MapperParams { self.mapper003.params() }
    fn params_mut(&mut self) -> &mut MapperParams { self.mapper003.params_mut() }
}

impl Mapper101 {
    pub fn new(cartridge: &Cartridge) -> Result<Mapper101, String> {
        Ok(Mapper101 {
            mapper003: Mapper003::new(cartridge)?,
        })
    }
}
