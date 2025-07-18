use crate::mapper::Cartridge;
use crate::mapper::{Mapper, MapperParams, LookupResult};
use crate::mappers as m;

pub fn lookup_mapper_with_params(cartridge: &Cartridge) -> (Box<dyn Mapper>, MapperParams) {
    let number = cartridge.mapper_number();
    let sub_number = cartridge.submapper_number();
    let cartridge_name = cartridge.name();

    let mapper = match lookup_mapper(cartridge) {
        LookupResult::Supported(supported_mapper) => supported_mapper,
        LookupResult::UnassignedMapper =>
            panic!("Mapper {number} is not in use. ROM: {cartridge_name}"),
        LookupResult::UnassignedSubmapper =>
            panic!("Submapper {} of mapper {number} is not in use. ROM: {cartridge_name}", sub_number.unwrap()),
        LookupResult::TodoMapper =>
            todo!("Mapper {number}. ROM: {cartridge_name}"),
        LookupResult::TodoSubmapper =>
            todo!("Submapper {}. ROM: {cartridge_name}", sub_number.unwrap()),
        LookupResult::UnspecifiedSubmapper =>
            panic!("Mapper {number} must have a submapper number with specified behavior. ROM: {cartridge_name}"),
        LookupResult::ReassignedSubmapper {correct_mapper, correct_submapper } =>
            panic!("Mapper {number}, submapper {} has been reassigned to {correct_mapper}, {correct_submapper} . ROM: {cartridge_name}", sub_number.unwrap()),
    };

    let mut mapper_params = mapper.layout().make_mapper_params(cartridge);
    mapper.init_mapper_params(&mut mapper_params);
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
            None => UnspecifiedSubmapper,
            // Normal behavior
            Some(0) => m::mapper001_0::Mapper001_0::new(cartridge).supported(),
            // SUROM, SOROM, SXROM
            Some(1 | 2 | 4) => ReassignedSubmapper { correct_mapper: 1, correct_submapper: 0 },
            Some(3) => ReassignedSubmapper { correct_mapper: 155, correct_submapper: 0 },
            // SEROM, SHROM, SH1ROM
            Some(5) => m::mapper001_5::Mapper001_5::default().supported(),
            // 2ME
            Some(6) => TodoSubmapper,
            _ => UnassignedSubmapper,
        }
        // UxROM
        2 => match submapper_number.unwrap_or(0) {
            0 => UnspecifiedSubmapper,
            // No bus conflicts
            1 => m::mapper002_1::MAPPER002_1.supported(),
            // Bus conflicts
            2 => m::mapper002_2::MAPPER002_2.supported(),
            _ => UnassignedSubmapper,
        }
        // CNROM
        3 => match submapper_number.unwrap_or(0) {
            0 => UnspecifiedSubmapper,
            // No bus conflicts
            1 => m::mapper003_1::MAPPER003_1.supported(),
            // Bus conflicts
            2 => m::mapper003_2::MAPPER003_2.supported(),
            _ => UnassignedSubmapper,
        }
        // MMC3
        4 => match submapper_number {
            None => UnspecifiedSubmapper,
            // Sharp IRQs
            Some(0) => m::mapper004_0::mapper004_0().supported(),
            // MMC6
            Some(1) => m::mapper004_1::Mapper004_1::new().supported(),
            Some(2) => UnassignedSubmapper,
            // MC-ACC IRQs
            Some(3) => m::mapper004_3::mapper004_3().supported(),
            // NEC IRQs
            Some(4) => m::mapper004_4::mapper004_4().supported(),
            // T9552 scrambling chip
            Some(5) => TodoSubmapper,
            // Rev A IRQ doesn't have a submapper assigned to it, despite being incompatible.
            Some(99) => m::mapper004_rev_a::mapper004_rev_a().supported(),
            _ => UnassignedSubmapper,
        }
        // MMC5
        5 => m::mapper005::Mapper005::new().supported(),

        // AxROM
        7 => match submapper_number.unwrap_or(0) {
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

        // K-1029 and K-1030P
        15 => m::mapper015::Mapper015.supported(),

        16 => match submapper_number.unwrap_or(0) {
            0 => UnspecifiedSubmapper,
            1 => ReassignedSubmapper { correct_mapper: 159, correct_submapper: 0 },
            2 => ReassignedSubmapper { correct_mapper: 157, correct_submapper: 0 },
            3 => ReassignedSubmapper { correct_mapper: 153, correct_submapper: 0 },
            // FCG-1
            4 => m::mapper016_4::Mapper016_4::default().supported(),
            // LZ93D50
            5 => m::mapper016_5::Mapper016_5::default().supported(),
            _ => UnassignedSubmapper,
        }

        // Jaleco SS 88006
        18 => m::mapper018::Mapper018::default().supported(),
        // Namco 129 and Namco 163.
        // (Expansion Audio isn't supported yet, so all submappers are the same for now.)
        19 => match submapper_number.unwrap_or(0) {
            0 => UnspecifiedSubmapper,
            // Duplicate of submapper 2.
            1 => m::mapper019::Mapper019::new().supported(),
            2 => m::mapper019::Mapper019::new().supported(),
            3 => m::mapper019::Mapper019::new().supported(),
            4 => m::mapper019::Mapper019::new().supported(),
            5 => m::mapper019::Mapper019::new().supported(),
            _ => UnassignedSubmapper,
        }
        // Famicom Disk System
        20 => panic!("Mapper 20 is only used for testing FDS images."),
        21 => match submapper_number.unwrap_or(0) {
            0 => UnspecifiedSubmapper,
            // VRC4a
            1 => m::mapper021_1::mapper021_1().supported(),
            // VRC4c
            2 => m::mapper021_2::mapper021_2().supported(),
            _ => UnassignedSubmapper,
        }
        // VRC2a
        22 => m::mapper022::mapper022().supported(),
        23 => match submapper_number.unwrap_or(0) {
            0 => UnspecifiedSubmapper,
            // VRC4f
            1 => m::mapper023_1::mapper023_1().supported(),
            // VRC4e
            2 => m::mapper023_2::mapper023_2().supported(),
            // VRC2b
            3 => m::mapper023_3::mapper023_3().supported(),
            _ => UnassignedSubmapper,
        }

        25 => match submapper_number.unwrap_or(0) {
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
            None => UnspecifiedSubmapper,
            // Normal behavior
            Some(0) => m::mapper032::Mapper032.supported(),
            // One-screen mirroring, fixed PRG banks (only Major League)
            Some(1) => TodoSubmapper,
            _ => UnassignedSubmapper,
        }
        // Taito's TC0190
        33 => m::mapper033::Mapper033.supported(),
        34 => match submapper_number.unwrap_or(0) {
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
        // NTDEC 2722 and NTDEC 2752 PCB and imitations
        40 => m::mapper040::Mapper040::default().supported(),
        // Caltron 6-in-1
        41 => m::mapper041::Mapper041::default().supported(),
        // FDS games hacked into cartridge form
        42 => m::mapper042::Mapper042::new(cartridge.chr_work_ram_size()).supported(),
        // TONY-I and YS-612 (FDS games in cartridge form)
        43 => m::mapper043::Mapper043::default().supported(),

        // Rumble Station
        46 => m::mapper046::Mapper046::default().supported(),
        // Super Spike V'Ball + Nintendo World Cup
        47 => m::mapper047::Mapper047::new().supported(),
        // Taito TC0690
        48 => m::mapper048::Mapper048::new().supported(),

        // N-32 conversion of Super Mario Bros. 2 (J). PCB code 761214.
        50 => m::mapper050::Mapper050::default().supported(),

        // BTL-MARIO1-MALEE2
        55 => m::mapper055::Mapper055.supported(),

        // NROM-/CNROM-based multicarts
        58 => m::mapper058::Mapper058.supported(),

        // NTDEC 0324 PCB
        61 => m::mapper061::Mapper061::new(cartridge.chr_work_ram_size()).supported(),
        // Super 700-in-1
        62 => m::mapper062::Mapper062.supported(),
        63 => match submapper_number {
            None => UnspecifiedSubmapper,
            // TH2291-3 and CH-011
            Some(0) => m::mapper063_0::Mapper063_0.supported(),
            // 82AB
            Some(1) => m::mapper063_1::Mapper063_1.supported(),
            _ => UnassignedSubmapper,
        }
        // RAMBO-1
        64 => m::mapper064::Mapper064::new().supported(),
        // Irem's H3001
        65 => m::mapper065::Mapper065::default().supported(),
        // GxROM (GNROM and MHROM)
        66 => m::mapper066::Mapper066.supported(),
        // Sunsoft-3
        67 => m::mapper067::Mapper067::default().supported(),

        // Sunsoft FME-7
        69 => m::mapper069::Mapper069::new().supported(),
        // Family Trainer and others
        70 => m::mapper070::Mapper070.supported(),
        // Codemasters
        71 => match submapper_number {
            None => UnspecifiedSubmapper,
            // Hardwired mirroring
            // FIXME: Implement specific submapper.
            Some(0) => m::mapper071::Mapper071.supported(),
            // Mapper-controlled mirroring (only Fire Hawk)
            // FIXME: Implement specific submapper.
            Some(1) => m::mapper071::Mapper071.supported(),
            _ => UnassignedSubmapper,
        }

        // VRC3
        73 => m::mapper073::Mapper073::default().supported(),

        // Konami VRC1
        75 => m::mapper075::Mapper075::default().supported(),
        // NAMCOT-3446
        76 => m::mapper076::Mapper076::new().supported(),
        // Irem (Napoleon Senki)
        77 => m::mapper077::Mapper077.supported(),
        78 => match submapper_number.unwrap_or(0) {
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
        // Taito's X1-005
        80 => m::mapper080::Mapper080.supported(),
        // Super Gun from NTDEC
        81 => m::mapper081::Mapper081.supported(),
        // Taito X1-017
        82 => m::mapper082::Mapper082.supported(),

        84 => UnassignedMapper,
        85 => match submapper_number.unwrap_or(0) {
            0 => UnspecifiedSubmapper,
            1 => m::mapper085_1::Mapper085_1::default().supported(),
            2 => m::mapper085_2::Mapper085_2::default().supported(),
            _ => UnassignedSubmapper,
        }
        // Jaleco JF-13
        86 => m::mapper086::Mapper086.supported(),
        // Jaleco J87
        87 => m::mapper087::Mapper087.supported(),
        // NAMCOT-3443
        88 => m::mapper088::Mapper088::new(cartridge).supported(),
        // Sunsoft (Tenka no Goikenban: Mito Koumon (J))
        89 => m::mapper089::Mapper089.supported(),

        // Sunsoft-2 IC on the Sunsoft-3R board
        93 => m::mapper093::Mapper093.supported(),
        // HVC-UN1ROM
        94 => m::mapper094::Mapper094.supported(),

        // Irem TAM-S1 (Kaiketsu Yanchamaru)
        97 => m::mapper097::Mapper097.supported(),
        98 => UnassignedMapper,

        // JF-10 misdump (only Urusei Yatsura - Lum no Wedding Bell)
        101 => m::mapper101::MAPPER101.supported(),
        102 => UnassignedMapper,

        // Magic Dragon
        107 => m::mapper107::Mapper107.supported(),

        // Huang Di and San Guo Zhi - Qun Xiong Zheng Ba
        112 => m::mapper112::Mapper112::new().supported(),
        // HES NTD-8
        113 => m::mapper113::Mapper113.supported(),

        // TxSROM
        118 => m::mapper118::Mapper118::new().supported(),
        // TQROM
        119 => m::mapper119::Mapper119::new().supported(),

        // Duplicate
        122 => m::mapper184::Mapper184.supported(),

        // Monty on the Run (Whirlwind Manu's FDS conversion)
        125 => m::mapper125::Mapper125.supported(),

        // Sachen 3009
        133 => m::mapper133::Mapper133.supported(),

        // Sachen 8259 B (UNL-Sachen-8259B)
        138 => m::mapper138::MAPPER138.supported(),
        // Sachen 8259 C (UNL-Sachen-8259C)
        139 => m::mapper139::MAPPER139.supported(),
        // Jaleco J-11 and J-14
        140 => m::mapper140::Mapper140.supported(),
        // Sachen 8259 A TC-A003-72 (UNL-Sachen-8259A)
        141 => m::mapper141::MAPPER141.supported(),

        // Kaiser KS202 (UNL-KS7032)
        142 => m::mapper142::Mapper142::default().supported(),

        // SA-72007 (only Sidewinder)
        145 => m::mapper145::Mapper145.supported(),
        // Duplicate of mapper 79, specifically for the Sachen 3015 board.
        146 => m::mapper079::Mapper079.supported(),

        // Sachen SA-008-A and Tengen 800008
        148 => m::mapper148::Mapper148.supported(),
        // Sachen SA-0036 (Taiwan Mahjong 16)
        149 => m::mapper149::Mapper149.supported(),

        // Duplicate
        151 => m::mapper075::Mapper075::default().supported(),
        // TAITO-74*161/161/32 and BANDAI-74*161/161/32
        152 => m::mapper152::Mapper152.supported(),

        // NAMCOT-3453 (only Devil Man)
        154 => m::mapper154::Mapper154::new(cartridge).supported(),

        // Almost a duplicate, but has different EEPROM behavior (not implemented yet).
        159 => m::mapper016_4::Mapper016_4::default().supported(),

        // Duplicate. Hanjuku Eiyuu (J).
        161 => m::mapper001_0::Mapper001_0::new(cartridge).supported(),

        // Hengedianzi (恒格电子) two-screen mirroring
        177 => m::mapper177::Mapper177.supported(),

        // UNROM 74HC08 (only Crazy Climber)
        180 => m::mapper180::Mapper180.supported(),
        181 => UnassignedMapper,

        // Sunsoft-1
        184 => m::mapper184::Mapper184.supported(),
        // CNROM with CHR RAM disable
        185 => match submapper_number {
            None => UnspecifiedSubmapper,
            Some(0) => m::mapper185_0::Mapper185_0::default().supported(),
            Some(4) => m::mapper185_4::MAPPER185_4.supported(),
            Some(5) => m::mapper185_5::MAPPER185_5.supported(),
            Some(6) => m::mapper185_6::MAPPER185_6.supported(),
            Some(7) => m::mapper185_7::MAPPER185_7.supported(),
            _ => UnassignedSubmapper,
        }
        // Used when running the BIOS of the Fukutake Study Box.
        186 => UnassignedMapper,

        // TXC-PT8154
        189 => m::mapper189::Mapper189::new().supported(),

        // NTDEC's TC-112
        193 => m::mapper193::Mapper193.supported(),

        // NROM-128 multicarts
        200 => match submapper_number {
            None => UnspecifiedSubmapper,
            // More PRG/CHR banks
            Some(0) => m::mapper200_0::Mapper200_0.supported(),
            // Fewer PRG/CHR banks
            Some(1) => m::mapper200_1::Mapper200_1.supported(),
            _ => UnassignedSubmapper,
        }
        // NROM-256 multicarts
        201 => m::mapper201::Mapper201.supported(),
        // 150-in-1 pirate cart
        202 => m::mapper202::Mapper202.supported(),
        // 35-in-1
        203 => m::mapper203::Mapper203.supported(),

        // DxROM, Tengen MIMIC-1, Namcot 118
        206 => match submapper_number {
            None => UnspecifiedSubmapper,
            // Normal PRG banking
            Some(0) => m::mapper206::Mapper206::new().supported(),
            // Fixed 32KiB PRG bank
            Some(1) => TodoSubmapper,
            _ => UnassignedSubmapper,
        }
        // Taito's X1-005 (alternate name table mirrorings)
        207 => m::mapper207::Mapper207::new().supported(),

        210 => match submapper_number.unwrap_or(0) {
            0 => UnspecifiedSubmapper,
            // Namco 175
            1 => m::mapper210_1::Mapper210_1.supported(),
            // Namco 340
            2 => m::mapper210_2::Mapper210_2.supported(),
            _ => UnassignedSubmapper,
        }

        // Duplicate
        213 => m::mapper058::Mapper058.supported(),

        // Quattro
        232 => match submapper_number {
            None => UnspecifiedSubmapper,
            // Normal behavior
            Some(0) => m::mapper232::Mapper232.supported(),
            // Aladdin Deck Enhancer
            Some(1) => TodoSubmapper,
            _ => UnassignedSubmapper,
        }

        // Maxi 15 multicart
        234 => m::mapper234::Mapper234::default().supported(),

        239 => UnassignedMapper,
        240 => m::mapper240::Mapper240.supported(),
        // Hengedianzi (恒格电子) hard-wired mirroring, and mapper hacks (m 164, 178, 227)
        241 => m::mapper241::Mapper241.supported(),

        247 => UnassignedMapper,

        _ => TodoMapper,
    }
}
