use crate::memory::mapper::Cartridge;
use crate::memory::mapper::{Mapper, MapperParams, InitialLayout};
use crate::memory::mappers as m;
use crate::memory::bank_index::BankIndexRegisterId::*;
use crate::memory::bank_index::MetaRegisterId::*;

pub fn lookup_mapper(cartridge: &Cartridge) -> (Box<dyn Mapper>, MapperParams) {
    let (mapper, initial_layout): (Box<dyn Mapper>, InitialLayout) = match cartridge.mapper_number() {
        0 => b(m::mapper000::Mapper000::new()),
        1 => b(m::mapper001::Mapper001::new()),
        2 => b(m::mapper002::Mapper002::new()),
        3 => b(m::mapper003::Mapper003::new()),
        4 => b(m::mapper004::Mapper004::new()),
        5 => b(m::mapper005::Mapper005::new()),

        7 => b(m::mapper007::Mapper007::new()),

        9 => b(m::mapper009::Mapper009::new()),
        10 => b(m::mapper010::Mapper010::new()),
        11 => b(m::mapper011::Mapper011::new()),

        32 => b(m::mapper032::Mapper032::new()),
        33 => b(m::mapper033::Mapper033::new()),
        34 => b(m::mapper034::Mapper034::new(cartridge)),

        66 => b(m::mapper066::Mapper066::new()),

        71 => b(m::mapper071::Mapper071::new()),

        87 => b(m::mapper087::Mapper087::new()),

        110 => b(m::mapper101::Mapper101::new()),

        140 => b(m::mapper140::Mapper140::new()),

        232 => b(m::mapper232::Mapper232::new()),
        _ => todo!(),
    };

    let mut mapper_params = initial_layout.make_mapper_params(cartridge);
    // FIXME: HACK
    if cartridge.mapper_number() == 10 {
        mapper_params.chr_memory_mut().set_meta_register(M0, C1);
        mapper_params.chr_memory_mut().set_meta_register(M1, C3);
    }

    (mapper, mapper_params)
}

fn b<M: Mapper + 'static>((mapper, layout): (M, InitialLayout)) -> (Box<dyn Mapper>, InitialLayout) {
    (Box::new(mapper), layout)
}
