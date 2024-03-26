use crate::memory::mapper::Cartridge;
use crate::memory::mapper::{Mapper, MapperParams};
use crate::memory::mappers as m;
use crate::memory::bank_index::BankIndexRegisterId::*;
use crate::memory::bank_index::MetaRegisterId::*;

pub fn lookup_mapper(cartridge: &Cartridge) -> (Box<dyn Mapper>, MapperParams) {
    let mapper: Box<dyn Mapper> = match (cartridge.mapper_number(), cartridge.submapper_number()) {
        // NROM
        (0, 0) => Box::new(m::mapper000::Mapper000),
        (1, _) => Box::new(m::mapper001::Mapper001::new()),
        (2, _) => Box::new(m::mapper002::Mapper002),
        (3, _) => Box::new(m::mapper003::Mapper003),
        (4, 0) => Box::new(m::mapper004::Mapper004::new()),
        (5, 0) => Box::new(m::mapper005::Mapper005::new()),

        (7, _) => Box::new(m::mapper007::Mapper007),

        (9, 0) => Box::new(m::mapper009::Mapper009),
        (10, 0) => Box::new(m::mapper010::Mapper010),
        (11, 0) => Box::new(m::mapper011::Mapper011),

        (13, 0) => Box::new(m::mapper013::Mapper013),

        // FCG-1
        (16, 4) => Box::new(m::mapper016_4::Mapper016_4::new()),
        // LZ93D50
        (16, 5) => Box::new(m::mapper016_5::Mapper016_5::new()),

        (21, 1) => m::mapper021_1::mapper021_1(),
        (21, 2) => m::mapper021_2::mapper021_2(),
        (22, 0) => m::mapper022::mapper022(),
        (23, 1) => m::mapper023_1::mapper023_1(),
        (23, 2) => m::mapper023_2::mapper023_2(),
        (23, 3) => m::mapper023_3::mapper023_3(),

        (25, 1) => m::mapper025_1::mapper025_1(),
        (25, 2) => m::mapper025_2::mapper025_2(),
        (25, 3) => m::mapper025_3::mapper025_3(),

        // Duplicate of 23, most likely.
        (27, 0) => m::mapper023_1::mapper023_1(),

        (32, 0) => Box::new(m::mapper032::Mapper032),
        (33, 0) => Box::new(m::mapper033::Mapper033),
        (34, 0) => Box::new(m::mapper034::Mapper034::new(cartridge)),

        (38, 0) => Box::new(m::mapper038::Mapper038),
        // Duplicate of 241.
        (39, 0) => Box::new(m::mapper039::Mapper039::new()),

        (46, 0) => Box::new(m::mapper046::Mapper046::new()),

        (64, 0) => Box::new(m::mapper064::Mapper064::new()),
        (65, 0) => Box::new(m::mapper065::Mapper065::new()),
        (66, 0) => Box::new(m::mapper066::Mapper066),

        (70, 0) => Box::new(m::mapper070::Mapper070),
        (71, _) => Box::new(m::mapper071::Mapper071),

        (75, 0) => Box::new(m::mapper075::Mapper075::new()),
        // NAMCOT-3446
        (76, 0) => Box::new(m::mapper076::Mapper076::new()),

        (87, 0) => Box::new(m::mapper087::Mapper087),
        (88, 0) => Box::new(m::mapper088::Mapper088::new(cartridge)),

        (94, 0) => Box::new(m::mapper094::Mapper094),

        (101, 0) => Box::new(m::mapper101::Mapper101::new()),

        (140, 0) => Box::new(m::mapper140::Mapper140),

        (152, 0) => Box::new(m::mapper152::Mapper152),

        (154, 0) => Box::new(m::mapper154::Mapper154::new(cartridge)),

        (177, 0) => Box::new(m::mapper177::Mapper177),

        (180, 0) => Box::new(m::mapper180::Mapper180),

        // DxROM, Tengen MIMIC-1, Namcot 118
        (206, 0) => Box::new(m::mapper206::Mapper206::new()),

        // Namco 175
        (210, 1) => Box::new(m::mapper210_1::Mapper210_1),
        // Namco 340
        (210, 2) => Box::new(m::mapper210_2::Mapper210_2),

        (232, 0) => Box::new(m::mapper232::Mapper232),

        (241, 0) => Box::new(m::mapper241::Mapper241),

        (m, s) => todo!("Mapper {m} submapper {s} isn't implemented yet."),
    };

    let mut mapper_params = mapper.initial_layout().make_mapper_params(cartridge);
    // FIXME: HACK
    if cartridge.mapper_number() == 10 {
        mapper_params.set_meta_register(M0, C1);
        mapper_params.set_meta_register(M1, C3);
    }

    (mapper, mapper_params)
}
