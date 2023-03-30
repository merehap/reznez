use crate::memory::mapper::Cartridge;
use crate::memory::mapper::Mapper;
use crate::memory::mappers as m;

pub fn lookup_mapper(cartridge: &Cartridge) -> Box<dyn Mapper> {
    match cartridge.mapper_number() {
        0 => Box::new(m::mapper000::Mapper000::new(cartridge).unwrap()) as Box<dyn Mapper>,
        1 => Box::new(m::mapper001::Mapper001::new(cartridge).unwrap()),
        2 => Box::new(m::mapper002::Mapper002::new(cartridge).unwrap()),
        3 => Box::new(m::mapper003::Mapper003::new(cartridge).unwrap()),
        4 => Box::new(m::mapper004::Mapper004::new(cartridge).unwrap()),
        5 => Box::new(m::mapper005::Mapper005::new(cartridge).unwrap()),

        7 => Box::new(m::mapper007::Mapper007::new(cartridge).unwrap()),

        9 => Box::new(m::mapper009::Mapper009::new(cartridge).unwrap()),
        10 => Box::new(m::mapper010::Mapper010::new(cartridge).unwrap()),
        11 => Box::new(m::mapper011::Mapper011::new(cartridge).unwrap()),

        66 => Box::new(m::mapper066::Mapper066::new(cartridge).unwrap()),

        71 => Box::new(m::mapper071::Mapper071::new(cartridge).unwrap()),

        87 => Box::new(m::mapper087::Mapper087::new(cartridge).unwrap()),

        110 => Box::new(m::mapper101::Mapper101::new(cartridge).unwrap()),

        140 => Box::new(m::mapper140::Mapper140::new(cartridge).unwrap()),

        232 => Box::new(m::mapper232::Mapper232::new(cartridge).unwrap()),
        _ => todo!(),
    }
}
