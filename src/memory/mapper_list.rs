use crate::memory::mapper::Cartridge;
use crate::memory::mapper::{Mapper, MapperParams, LookupResult};
use crate::memory::mappers as m;

pub fn lookup_mapper_with_params(cartridge: &Cartridge) -> (Box<dyn Mapper>, MapperParams) {
    let number = cartridge.mapper_number();
    let sub_number = cartridge.submapper_number();
    let cartridge_name = cartridge.name();

    let mapper = match lookup_mapper(cartridge) {
        LookupResult::Supported(supported_mapper) => supported_mapper,
        LookupResult::UnassignedMapper =>
            panic!("Mapper {number} is not in use. ROM: {cartridge_name}"),
        LookupResult::UnassignedSubmapper =>
            panic!("Submapper {sub_number} of mapper {number} is not in use. ROM: {cartridge_name}"),
        LookupResult::TodoMapper =>
            todo!("Mapper {number}. ROM: {cartridge_name}"),
        LookupResult::TodoSubmapper =>
            todo!("Submapper {sub_number}. ROM: {cartridge_name}"),
        LookupResult::UnspecifiedSubmapper =>
            panic!("Mapper {number}, submapper {sub_number} has unspecified behavior. ROM: {cartridge_name}"),
        LookupResult::ReassignedSubmapper {correct_mapper, correct_submapper } =>
            panic!("Mapper {number}, submapper {sub_number} has been reassigned to {correct_mapper}, {correct_submapper} ."),
    };

    let mapper_params = mapper.initial_layout().make_mapper_params(cartridge);
    (mapper, mapper_params)
}

fn lookup_mapper(cartridge: &Cartridge) -> LookupResult {
    use LookupResult::*;
    let submapper_number = cartridge.submapper_number();
    match cartridge.mapper_number() {
        // NROM
        0 => m::mapper000::Mapper000.supported(),
        1 => match submapper_number {
            0 => m::mapper001_0::Mapper001_0::new(cartridge).supported(),
            1 | 2 | 4 => ReassignedSubmapper { correct_mapper: 1, correct_submapper: 0 },
            3 => ReassignedSubmapper { correct_mapper: 155, correct_submapper: 0 },
            5 => m::mapper001_5::Mapper001_5::new().supported(),
            6 => TodoSubmapper,
            _ => UnassignedSubmapper,
        }
        2 => match submapper_number {
            0 => UnspecifiedSubmapper,
            1 => m::mapper002_1::MAPPER002_1.supported(),
            2 => m::mapper002_2::MAPPER002_2.supported(),
            _ => UnassignedSubmapper,
        }
        3 => match submapper_number {
            0 => UnspecifiedSubmapper,
            1 => m::mapper003_1::MAPPER003_1.supported(),
            2 => m::mapper003_2::MAPPER003_2.supported(),
            _ => UnassignedSubmapper,
        }
        4 => match submapper_number {
            0 => m::mapper004_0::mapper004_0().supported(),
            1 => m::mapper004_1::Mapper004_1::new().supported(),
            2 => UnassignedSubmapper,
            3 => m::mapper004_3::mapper004_3().supported(),
            4 => m::mapper004_4::mapper004_4().supported(),
            // T9552 scrambling chip
            5 => TodoSubmapper,
            // Rev A IRQ doesn't have a submapper assigned to it, despite being incompatible.
            99 => m::mapper004_rev_a::mapper004_rev_a().supported(),
            _ => UnassignedSubmapper,
        }
        5 => m::mapper005::Mapper005::new().supported(),

        7 => match submapper_number {
            0 => UnspecifiedSubmapper,
            1 => m::mapper007_1::MAPPER007_1.supported(),
            2 => m::mapper007_2::MAPPER007_2.supported(),
            _ => UnassignedSubmapper,
        }

        9 => m::mapper009::Mapper009.supported(),
        10 => m::mapper010::Mapper010.supported(),
        11 => m::mapper011::Mapper011.supported(),

        13 => m::mapper013::Mapper013.supported(),

        16 => match submapper_number {
            0 => UnspecifiedSubmapper,
            1 => ReassignedSubmapper { correct_mapper: 159, correct_submapper: 0 },
            2 => ReassignedSubmapper { correct_mapper: 157, correct_submapper: 0 },
            3 => ReassignedSubmapper { correct_mapper: 153, correct_submapper: 0 },
            // FCG-1
            4 => m::mapper016_4::Mapper016_4::new().supported(),
            // LZ93D50
            5 => m::mapper016_5::Mapper016_5::new().supported(),
            _ => UnassignedSubmapper,
        }

        18 => m::mapper018::Mapper018::new().supported(),

        20 => panic!("Mapper 20 is only used for testing FDS images."),
        21 => match submapper_number {
            0 => UnspecifiedSubmapper,
            1 => m::mapper021_1::mapper021_1().supported(),
            2 => m::mapper021_2::mapper021_2().supported(),
            _ => UnassignedSubmapper,
        }
        22 => m::mapper022::mapper022().supported(),
        23 => match submapper_number {
            0 => UnspecifiedSubmapper,
            1 => m::mapper023_1::mapper023_1().supported(),
            2 => m::mapper023_2::mapper023_2().supported(),
            3 => m::mapper023_3::mapper023_3().supported(),
            _ => UnassignedSubmapper,
        }

        25 => match submapper_number {
            0 => UnspecifiedSubmapper,
            1 => m::mapper025_1::mapper025_1().supported(),
            2 => m::mapper025_2::mapper025_2().supported(),
            3 => m::mapper025_3::mapper025_3().supported(),
            _ => UnassignedSubmapper,
        }

        // Duplicate of 23, most likely.
        27 => m::mapper023_1::mapper023_1().supported(),

        32 => match submapper_number {
            0 => m::mapper032::Mapper032.supported(),
            1 => TodoSubmapper,
            _ => UnassignedSubmapper,
        }
        33 => m::mapper033::Mapper033.supported(),
        34 => match submapper_number {
            0 => UnspecifiedSubmapper,
            1 => m::mapper034_1::Mapper034_1.supported(),
            2 => m::mapper034_2::Mapper034_2.supported(),
            _ => UnassignedSubmapper,
        }

        38 => m::mapper038::Mapper038.supported(),
        // Duplicate of 241.
        39 => m::mapper039::Mapper039::new().supported(),

        46 => m::mapper046::Mapper046::new().supported(),

        64 => m::mapper064::Mapper064::new().supported(),
        65 => m::mapper065::Mapper065::new().supported(),
        66 => m::mapper066::Mapper066.supported(),

        69 => m::mapper069::Mapper069::new().supported(),
        70 => m::mapper070::Mapper070.supported(),
        71 => match submapper_number {
            // FIXME: Implement specific submapper.
            0 => m::mapper071::Mapper071.supported(),
            // FIXME: Implement specific submapper.
            1 => m::mapper071::Mapper071.supported(),
            _ => UnassignedSubmapper,
        }

        75 => m::mapper075::Mapper075::new().supported(),
        // NAMCOT-3446
        76 => m::mapper076::Mapper076::new().supported(),

        78 => match submapper_number {
            0 => UnspecifiedSubmapper,
            1 => m::mapper078_1::Mapper078_1.supported(),
            2 => UnassignedSubmapper,
            3 => m::mapper078_3::Mapper078_3.supported(),
            _ => UnassignedSubmapper,
        }
        79 => m::mapper079::Mapper079.supported(),

        86 => m::mapper086::Mapper086.supported(),
        87 => m::mapper087::Mapper087.supported(),
        88 => m::mapper088::Mapper088::new(cartridge).supported(),

        94 => m::mapper094::Mapper094.supported(),

        98 => UnassignedMapper,

        101 => m::mapper101::MAPPER101.supported(),

        113 => m::mapper113::Mapper113.supported(),

        133 => m::mapper133::Mapper133.supported(),

        140 => m::mapper140::Mapper140.supported(),

        145 => m::mapper145::Mapper145.supported(),
        // Duplicate of mapper 79, specifically for the Sachen 3015 board.
        146 => m::mapper079::Mapper079.supported(),

        148 => m::mapper148::Mapper148.supported(),
        149 => m::mapper149::Mapper149.supported(),

        152 => m::mapper152::Mapper152.supported(),

        154 => m::mapper154::Mapper154::new(cartridge).supported(),

        177 => m::mapper177::Mapper177.supported(),

        180 => m::mapper180::Mapper180.supported(),

        // DxROM, Tengen MIMIC-1, Namcot 118
        206 => match submapper_number {
            0 => m::mapper206::Mapper206::new().supported(),
            1 => TodoSubmapper,
            _ => UnassignedSubmapper,
        }

        210 => match submapper_number {
            0 => UnspecifiedSubmapper,
            // Namco 175
            1 => m::mapper210_1::Mapper210_1.supported(),
            // Namco 340
            2 => m::mapper210_2::Mapper210_2.supported(),
            _ => UnassignedSubmapper,
        }

        232 => match submapper_number {
            0 => m::mapper232::Mapper232.supported(),
            1 => TodoSubmapper,
            _ => UnassignedSubmapper,
        }

        239 => UnassignedMapper,
        240 => m::mapper240::Mapper240.supported(),
        241 => m::mapper241::Mapper241.supported(),

        247 => UnassignedMapper,

        _ => TodoMapper,
    }
}
