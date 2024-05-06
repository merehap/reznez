use crate::memory::mapper::Cartridge;
use crate::memory::mapper::{Mapper, MapperParams};
use crate::memory::mappers as m;

pub fn lookup_mapper_with_params(cartridge: &Cartridge) -> (Box<dyn Mapper>, MapperParams) {
    let number = cartridge.mapper_number();
    let sub_number = cartridge.submapper_number();
    let cartridge_name = cartridge.name();

    let mapper;
    use LookupResult::*;
    match lookup_mapper(cartridge) {
        Supported(supported_mapper) =>
            mapper = supported_mapper,
        UnassignedMapper =>
            panic!("Mapper {number} is not in use. ROM: {cartridge_name}"),
        UnassignedSubmapper =>
            panic!("Submapper {sub_number} of mapper {number} is not in use. ROM: {cartridge_name}"),
        TodoMapper =>
            todo!("Mapper {number}. ROM: {cartridge_name}"),
        TodoSubmapper =>
            todo!("Submapper {sub_number}. ROM: {cartridge_name}"),
        UnspecifiedSubmapper =>
            panic!("Mapper {number}, submapper {sub_number} has unspecified behavior. ROM: {cartridge_name}"),
        ReassignedSubmapper {correct_mapper, correct_submapper } =>
            panic!("Mapper {number}, submapper {sub_number} has been reassigned to {correct_mapper}, {correct_submapper} ."),
    }

    let mapper_params = mapper.initial_layout().make_mapper_params(cartridge);
    (mapper, mapper_params)
}

fn lookup_mapper(cartridge: &Cartridge) -> LookupResult {
    let submapper_number = cartridge.submapper_number();
    LookupResult::Supported(match cartridge.mapper_number() {
        // NROM
        0 => Box::new(m::mapper000::Mapper000),
        1 => match submapper_number {
            0 => Box::new(m::mapper001_0::Mapper001_0::new()),
            1 | 2 | 4 => return LookupResult::ReassignedSubmapper { correct_mapper: 1, correct_submapper: 0 },
            3 => return LookupResult::ReassignedSubmapper { correct_mapper: 155, correct_submapper: 0 },
            5 => Box::new(m::mapper001_5::Mapper001_5::new()),
            6 => return LookupResult::TodoSubmapper,
            _ => return LookupResult::UnassignedSubmapper,
        }
        2 => match submapper_number {
            0 => return LookupResult::UnspecifiedSubmapper,
            1 => Box::new(m::mapper002_1::MAPPER002_1),
            2 => Box::new(m::mapper002_2::MAPPER002_2),
            _ => return LookupResult::UnassignedSubmapper,
        }
        3 => match submapper_number {
            0 => return LookupResult::UnspecifiedSubmapper,
            1 => Box::new(m::mapper003_1::MAPPER003_1),
            2 => Box::new(m::mapper003_2::MAPPER003_2),
            _ => return LookupResult::UnassignedSubmapper,
        }
        4 => match submapper_number {
            0 => Box::new(m::mapper004_0::mapper004_0()),
            1 => Box::new(m::mapper004_1::Mapper004_1::new()),
            2 => return LookupResult::UnassignedSubmapper,
            3 => Box::new(m::mapper004_3::mapper004_3()),
            4 => Box::new(m::mapper004_4::mapper004_4()),
            // T9552 scrambling chip
            5 => return LookupResult::TodoSubmapper,
            // Rev A IRQ doesn't have a submapper assigned to it, despite being incompatible.
            99 => Box::new(m::mapper004_rev_a::mapper004_rev_a()),
            _ => return LookupResult::UnassignedSubmapper,
        }
        5 => Box::new(m::mapper005::Mapper005::new()),

        7 => match submapper_number {
            0 => return LookupResult::UnspecifiedSubmapper,
            1 => Box::new(m::mapper007_1::MAPPER007_1),
            2 => Box::new(m::mapper007_2::MAPPER007_2),
            _ => return LookupResult::UnassignedSubmapper,
        }

        9 => Box::new(m::mapper009::Mapper009),
        10 => Box::new(m::mapper010::Mapper010),
        11 => Box::new(m::mapper011::Mapper011),

        13 => Box::new(m::mapper013::Mapper013),

        16 => match submapper_number {
            0 => return LookupResult::UnspecifiedSubmapper,
            1 => return LookupResult::ReassignedSubmapper { correct_mapper: 159, correct_submapper: 0 },
            2 => return LookupResult::ReassignedSubmapper { correct_mapper: 157, correct_submapper: 0 },
            3 => return LookupResult::ReassignedSubmapper { correct_mapper: 153, correct_submapper: 0 },
            // FCG-1
            4 => Box::new(m::mapper016_4::Mapper016_4::new()),
            // LZ93D50
            5 => Box::new(m::mapper016_5::Mapper016_5::new()),
            _ => return LookupResult::UnassignedSubmapper,
        }

        18 => Box::new(m::mapper018::Mapper018::new()),

        20 => panic!("Mapper 20 is only used for testing FDS images."),
        21 => match submapper_number {
            0 => return LookupResult::UnspecifiedSubmapper,
            1 => m::mapper021_1::mapper021_1(),
            2 => m::mapper021_2::mapper021_2(),
            _ => return LookupResult::UnassignedSubmapper,
        }
        22 => m::mapper022::mapper022(),
        23 => match submapper_number {
            0 => return LookupResult::UnspecifiedSubmapper,
            1 => m::mapper023_1::mapper023_1(),
            2 => m::mapper023_2::mapper023_2(),
            3 => m::mapper023_3::mapper023_3(),
            _ => return LookupResult::UnassignedSubmapper,
        }

        25 => match submapper_number {
            0 => return LookupResult::UnspecifiedSubmapper,
            1 => m::mapper025_1::mapper025_1(),
            2 => m::mapper025_2::mapper025_2(),
            3 => m::mapper025_3::mapper025_3(),
            _ => return LookupResult::UnassignedSubmapper,
        }

        // Duplicate of 23, most likely.
        27 => m::mapper023_1::mapper023_1(),

        32 => match submapper_number {
            0 => Box::new(m::mapper032::Mapper032),
            1 => return LookupResult::TodoSubmapper,
            _ => return LookupResult::UnassignedSubmapper,
        }
        33 => Box::new(m::mapper033::Mapper033),
        34 => match submapper_number {
            0 => return LookupResult::UnspecifiedSubmapper,
            1 => Box::new(m::mapper034_1::Mapper034_1),
            2 => Box::new(m::mapper034_2::Mapper034_2),
            _ => return LookupResult::UnassignedSubmapper,
        }

        38 => Box::new(m::mapper038::Mapper038),
        // Duplicate of 241.
        39 => Box::new(m::mapper039::Mapper039::new()),

        46 => Box::new(m::mapper046::Mapper046::new()),

        64 => Box::new(m::mapper064::Mapper064::new()),
        65 => Box::new(m::mapper065::Mapper065::new()),
        66 => Box::new(m::mapper066::Mapper066),

        69 => Box::new(m::mapper069::Mapper069::new()),
        70 => Box::new(m::mapper070::Mapper070),
        71 => match submapper_number {
            // FIXME: Implement specific submapper.
            0 => Box::new(m::mapper071::Mapper071),
            // FIXME: Implement specific submapper.
            1 => Box::new(m::mapper071::Mapper071),
            _ => return LookupResult::UnassignedSubmapper,
        }

        75 => Box::new(m::mapper075::Mapper075::new()),
        // NAMCOT-3446
        76 => Box::new(m::mapper076::Mapper076::new()),

        78 => match submapper_number {
            0 => return LookupResult::UnspecifiedSubmapper,
            1 => Box::new(m::mapper078_1::Mapper078_1),
            2 => return LookupResult::UnassignedSubmapper,
            3 => Box::new(m::mapper078_3::Mapper078_3),
            _ => return LookupResult::UnassignedSubmapper,
        }
        79 => Box::new(m::mapper079::Mapper079),

        86 => Box::new(m::mapper086::Mapper086),
        87 => Box::new(m::mapper087::Mapper087),
        88 => Box::new(m::mapper088::Mapper088::new(cartridge)),

        94 => Box::new(m::mapper094::Mapper094),

        98 => return LookupResult::UnassignedMapper,

        101 => Box::new(m::mapper101::MAPPER101),

        113 => Box::new(m::mapper113::Mapper113),

        133 => Box::new(m::mapper133::Mapper133),

        140 => Box::new(m::mapper140::Mapper140),

        145 => Box::new(m::mapper145::Mapper145),
        // Duplicate of mapper 79, specifically for the Sachen 3015 board.
        146 => Box::new(m::mapper079::Mapper079),

        148 => Box::new(m::mapper148::Mapper148),
        149 => Box::new(m::mapper149::Mapper149),

        152 => Box::new(m::mapper152::Mapper152),

        154 => Box::new(m::mapper154::Mapper154::new(cartridge)),

        177 => Box::new(m::mapper177::Mapper177),

        180 => Box::new(m::mapper180::Mapper180),

        // DxROM, Tengen MIMIC-1, Namcot 118
        206 => match submapper_number {
            0 => Box::new(m::mapper206::Mapper206::new()),
            1 => return LookupResult::TodoSubmapper,
            _ => return LookupResult::UnassignedSubmapper,
        }

        210 => match submapper_number {
            0 => return LookupResult::UnspecifiedSubmapper,
            // Namco 175
            1 => Box::new(m::mapper210_1::Mapper210_1),
            // Namco 340
            2 => Box::new(m::mapper210_2::Mapper210_2),
            _ => return LookupResult::UnassignedSubmapper,
        }

        232 => match submapper_number {
            0 => Box::new(m::mapper232::Mapper232),
            1 => return LookupResult::TodoSubmapper,
            _ => return LookupResult::UnassignedSubmapper,
        }

        239 => return LookupResult::UnassignedMapper,
        240 => Box::new(m::mapper240::Mapper240),
        241 => Box::new(m::mapper241::Mapper241),

        247 => return LookupResult::UnassignedMapper,

        _ => return LookupResult::TodoMapper,
    })
}

enum LookupResult {
    Supported(Box<dyn Mapper>),
    UnassignedMapper,
    UnassignedSubmapper,
    TodoMapper,
    TodoSubmapper,
    UnspecifiedSubmapper,
    ReassignedSubmapper {correct_mapper: u16, correct_submapper: u8 },
}
