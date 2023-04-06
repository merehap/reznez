use crate::memory::mapper::Cartridge;
use crate::memory::mapper::{Mapper, MapperParams};
use crate::memory::mappers as m;
use crate::memory::bank_index::BankIndexRegisterId::*;
use crate::memory::bank_index::MetaRegisterId::*;

pub fn lookup_mapper(cartridge: &Cartridge) -> (Box<dyn Mapper>, MapperParams) {
    let mapper: Box<dyn Mapper> = match cartridge.mapper_number() {
        0 => Box::new(m::mapper000::Mapper000),
        1 => Box::new(m::mapper001::Mapper001::new()),
        2 => Box::new(m::mapper002::Mapper002),
        3 => Box::new(m::mapper003::Mapper003),
        4 => Box::new(m::mapper004::Mapper004::new()),
        5 => Box::new(m::mapper005::Mapper005::new()),

        7 => Box::new(m::mapper007::Mapper007),

        9 => Box::new(m::mapper009::Mapper009),
        10 => Box::new(m::mapper010::Mapper010),
        11 => Box::new(m::mapper011::Mapper011),

        32 => Box::new(m::mapper032::Mapper032),
        33 => Box::new(m::mapper033::Mapper033),
        34 => Box::new(m::mapper034::Mapper034::new(cartridge)),

        66 => Box::new(m::mapper066::Mapper066),

        70 => Box::new(m::mapper070::Mapper070),
        71 => Box::new(m::mapper071::Mapper071),

        87 => Box::new(m::mapper087::Mapper087),

        101 => Box::new(m::mapper101::Mapper101::new()),

        140 => Box::new(m::mapper140::Mapper140),

        152 => Box::new(m::mapper152::Mapper152),

        232 => Box::new(m::mapper232::Mapper232),
        _ => todo!(),
    };

    let mut mapper_params = mapper.initial_layout().make_mapper_params(cartridge);
    // FIXME: HACK
    if cartridge.mapper_number() == 10 {
        mapper_params.chr_memory_mut().set_meta_register(M0, C1);
        mapper_params.chr_memory_mut().set_meta_register(M1, C3);
    }

    (mapper, mapper_params)
}
