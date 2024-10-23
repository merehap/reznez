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
        // MMC1
        1 => match submapper_number {
            // Normal behavior
            0 => m::mapper001_0::Mapper001_0::new(cartridge).supported(),
            // SUROM, SOROM, SXROM
            1 | 2 | 4 => ReassignedSubmapper { correct_mapper: 1, correct_submapper: 0 },
            3 => ReassignedSubmapper { correct_mapper: 155, correct_submapper: 0 },
            // SEROM, SHROM, SH1ROM
            5 => m::mapper001_5::Mapper001_5::new().supported(),
            // 2ME
            6 => TodoSubmapper,
            _ => UnassignedSubmapper,
        }
        // UxROM
        2 => match submapper_number {
            0 => UnspecifiedSubmapper,
            // No bus conflicts
            1 => m::mapper002_1::MAPPER002_1.supported(),
            // Bus conflicts
            2 => m::mapper002_2::MAPPER002_2.supported(),
            _ => UnassignedSubmapper,
        }
        // CNROM
        3 => match submapper_number {
            0 => UnspecifiedSubmapper,
            // No bus conflicts
            1 => m::mapper003_1::MAPPER003_1.supported(),
            // Bus conflicts
            2 => m::mapper003_2::MAPPER003_2.supported(),
            _ => UnassignedSubmapper,
        }
        // MMC3
        4 => match submapper_number {
            // Sharp IRQs
            0 => m::mapper004_0::mapper004_0().supported(),
            // MMC6
            1 => m::mapper004_1::Mapper004_1::new().supported(),
            2 => UnassignedSubmapper,
            // MC-ACC IRQs
            3 => m::mapper004_3::mapper004_3().supported(),
            // NEC IRQs
            4 => m::mapper004_4::mapper004_4().supported(),
            // T9552 scrambling chip
            5 => TodoSubmapper,
            // Rev A IRQ doesn't have a submapper assigned to it, despite being incompatible.
            99 => m::mapper004_rev_a::mapper004_rev_a().supported(),
            _ => UnassignedSubmapper,
        }
        // MMC5
        5 => m::mapper005::Mapper005::new().supported(),

        // AxROM
        7 => match submapper_number {
            0 => UnspecifiedSubmapper,
            // No bus conflicts
            1 => m::mapper007_1::MAPPER007_1.supported(),
            // Bus conflicts
            2 => m::mapper007_2::MAPPER007_2.supported(),
            _ => UnassignedSubmapper,
        }

        // MMC2
        9 => m::mapper009::Mapper009.supported(),
        // MMC4
        10 => m::mapper010::Mapper010.supported(),
        // Color Dreams
        11 => m::mapper011::Mapper011.supported(),

        // NES CPROM
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

        // Jaleco SS 88006
        18 => m::mapper018::Mapper018::new().supported(),

        // Famicom Disk System
        20 => panic!("Mapper 20 is only used for testing FDS images."),
        21 => match submapper_number {
            0 => UnspecifiedSubmapper,
            // VRC4a
            1 => m::mapper021_1::mapper021_1().supported(),
            // VRC4c
            2 => m::mapper021_2::mapper021_2().supported(),
            _ => UnassignedSubmapper,
        }
        // VRC2a
        22 => m::mapper022::mapper022().supported(),
        23 => match submapper_number {
            0 => UnspecifiedSubmapper,
            // VRC4f
            1 => m::mapper023_1::mapper023_1().supported(),
            // VRC4e
            2 => m::mapper023_2::mapper023_2().supported(),
            // VRC2b
            3 => m::mapper023_3::mapper023_3().supported(),
            _ => UnassignedSubmapper,
        }

        25 => match submapper_number {
            0 => UnspecifiedSubmapper,
            // VRC4b
            1 => m::mapper025_1::mapper025_1().supported(),
            // VRC4d
            2 => m::mapper025_2::mapper025_2().supported(),
            // VRC2c
            3 => m::mapper025_3::mapper025_3().supported(),
            _ => UnassignedSubmapper,
        }

        // Duplicate of 23, most likely.
        27 => m::mapper023_1::mapper023_1().supported(),

        // Homebrew. Sealie Computing - RET-CUFROM revD
        29 => m::mapper029::Mapper029.supported(),

        // Irem G101
        32 => match submapper_number {
            // Normal behavior
            0 => m::mapper032::Mapper032.supported(),
            // One-screen mirroring, fixed PRG banks (only Major League)
            1 => TodoSubmapper,
            _ => UnassignedSubmapper,
        }
        // Taito's TC0190
        33 => m::mapper033::Mapper033.supported(),
        34 => match submapper_number {
            0 => UnspecifiedSubmapper,
            // NINA-001
            1 => m::mapper034_1::Mapper034_1.supported(),
            // BNROM
            2 => m::mapper034_2::Mapper034_2.supported(),
            _ => UnassignedSubmapper,
        }

        // Bit Corp.'s Crime Busters
        38 => m::mapper038::Mapper038.supported(),
        // Duplicate of 241.
        39 => m::mapper039::Mapper039::new().supported(),

        // Rumble Station
        46 => m::mapper046::Mapper046::new().supported(),

        // RAMBO-1
        64 => m::mapper064::Mapper064::new().supported(),
        // Irem's H3001
        65 => m::mapper065::Mapper065::new().supported(),
        // GxROM (GNROM and MHROM)
        66 => m::mapper066::Mapper066.supported(),

        // Sunsoft FME-7
        69 => m::mapper069::Mapper069::new().supported(),
        // Family Trainer and others
        70 => m::mapper070::Mapper070.supported(),
        // Codemasters
        71 => match submapper_number {
            // Hardwired mirroring
            // FIXME: Implement specific submapper.
            0 => m::mapper071::Mapper071.supported(),
            // Mapper-controlled mirroring (only Fire Hawk)
            // FIXME: Implement specific submapper.
            1 => m::mapper071::Mapper071.supported(),
            _ => UnassignedSubmapper,
        }

        // Konami VRC1
        75 => m::mapper075::Mapper075::new().supported(),
        // NAMCOT-3446
        76 => m::mapper076::Mapper076::new().supported(),

        78 => match submapper_number {
            0 => UnspecifiedSubmapper,
            // Single-screen mirroring (only Cosmo Carrier)
            1 => m::mapper078_1::Mapper078_1.supported(),
            2 => UnassignedSubmapper,
            // Mapper-controlled mirroring (only Holy Diver)
            3 => m::mapper078_3::Mapper078_3.supported(),
            _ => UnassignedSubmapper,
        }
        // NINA-03 and NINA-06
        79 => m::mapper079::Mapper079.supported(),

        // Jaleco's JF-13
        86 => m::mapper086::Mapper086.supported(),
        // Jaleco J87
        87 => m::mapper087::Mapper087.supported(),
        // NAMCOT-3443
        88 => m::mapper088::Mapper088::new(cartridge).supported(),

        // HVC-UN1ROM
        94 => m::mapper094::Mapper094.supported(),

        98 => UnassignedMapper,

        // JF-10 misdump (only Urusei Yatsura - Lum no Wedding Bell)
        101 => m::mapper101::MAPPER101.supported(),

        // HES NTD-8
        113 => m::mapper113::Mapper113.supported(),

        // Sachen 3009
        133 => m::mapper133::Mapper133.supported(),

        // Jaleco J-11 and J-14
        140 => m::mapper140::Mapper140.supported(),

        // SA-72007 (only Sidewinder)
        145 => m::mapper145::Mapper145.supported(),
        // Duplicate of mapper 79, specifically for the Sachen 3015 board.
        146 => m::mapper079::Mapper079.supported(),

        // Sachen SA-008-A and Tengen 800008
        148 => m::mapper148::Mapper148.supported(),
        // Sachen SA-0036 (Taiwan Mahjong 16)
        149 => m::mapper149::Mapper149.supported(),

        // TAITO-74*161/161/32 and BANDAI-74*161/161/32
        152 => m::mapper152::Mapper152.supported(),

        // NAMCOT-3453 (only Devil Man)
        154 => m::mapper154::Mapper154::new(cartridge).supported(),

        // Hengedianzi (恒格电子) two-screen mirroring
        177 => m::mapper177::Mapper177.supported(),

        // UNROM 74HC08 (only Crazy Climber)
        180 => m::mapper180::Mapper180.supported(),

        // DxROM, Tengen MIMIC-1, Namcot 118
        206 => match submapper_number {
            // Normal PRG banking
            0 => m::mapper206::Mapper206::new().supported(),
            // Fixed 32KiB PRG bank
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

        // Quattro
        232 => match submapper_number {
            // Normal behavior
            0 => m::mapper232::Mapper232.supported(),
            // Aladdin Deck Enhancer
            1 => TodoSubmapper,
            _ => UnassignedSubmapper,
        }

        239 => UnassignedMapper,
        240 => m::mapper240::Mapper240.supported(),
        // Hengedianzi (恒格电子) hard-wired mirroring, and mapper hacks (m 164, 178, 227)
        241 => m::mapper241::Mapper241.supported(),

        247 => UnassignedMapper,

        _ => TodoMapper,
    }
}
