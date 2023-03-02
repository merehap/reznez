use crate::memory::mapper::Cartridge;
use crate::memory::mapper::Mapper;
use crate::memory::mappers as m;

pub fn lookup_mapper(cartridge: &Cartridge) -> Box<dyn Mapper> {
    match cartridge.mapper_number() {
        000 => Box::new(m::mapper000::Mapper000::new(cartridge).unwrap()) as Box<dyn Mapper>,
        001 => Box::new(m::mapper001::Mapper001::new(cartridge).unwrap()),
        002 => Box::new(m::mapper002::Mapper002::new(cartridge).unwrap()),
        003 => Box::new(m::mapper003::Mapper003::new(cartridge).unwrap()),
        004 => Box::new(m::mapper004::Mapper004::new(cartridge).unwrap()),
        005 => Box::new(m::mapper005::Mapper005::new(cartridge).unwrap()),

        007 => Box::new(m::mapper007::Mapper007::new(cartridge).unwrap()),

        011 => Box::new(m::mapper011::Mapper011::new(cartridge).unwrap()),

        066 => Box::new(m::mapper066::Mapper066::new(cartridge).unwrap()),

        140 => Box::new(m::mapper140::Mapper140::new(cartridge).unwrap()),
        _ => todo!(),
    }
}
