use std::collections::BTreeSet;
use std::sync::LazyLock;

use crate::cartridge::resolved_metadata::{MetadataResolver, ResolvedMetadata};
use crate::mapper::Cartridge;
use crate::mapper::{Mapper, LookupResult};
use crate::mappers as m;

pub static MAPPERS_WITHOUT_SUBMAPPER_0: LazyLock<BTreeSet<u16>> = LazyLock::new(|| {
    (0..u16::MAX)
        .filter(|&mapper_number| {
            let metadata = ResolvedMetadata { mapper_number, submapper_number: Some(0), .. ResolvedMetadata::default()};
            matches!(try_lookup_mapper(&metadata), LookupResult::UnassignedSubmapper | LookupResult::UnspecifiedSubmapper)
        })
        .collect()
});

pub fn lookup_mapper(metadata_resolver: &MetadataResolver, cartridge: &Cartridge) -> Result<Box<dyn Mapper>, String> {
    let metadata = metadata_resolver.resolve();
    let number = metadata.mapper_number;
    let sub_number = metadata.submapper_number;
    let cartridge_name = cartridge.name();

    match try_lookup_mapper(&metadata) {
        LookupResult::Supported(supported_mapper) => Ok(supported_mapper),
        LookupResult::UnassignedMapper =>
            Err(format!("Mapper {number} is not in use. ROM: {cartridge_name}")),
        LookupResult::UnassignedSubmapper =>
            Err(format!("Submapper {} of mapper {number} is not in use. ROM: {cartridge_name}", sub_number.unwrap())),
        LookupResult::TodoMapper =>
            Err(format!("Mapper {number}. ROM: {cartridge_name}")),
        LookupResult::TodoSubmapper =>
            Err(format!("Submapper {}. ROM: {cartridge_name}", sub_number.unwrap())),
        LookupResult::UnspecifiedSubmapper =>
            Err(format!("Submapper {sub_number:?} of mapper {number} has unspecified behavior. ROM: {cartridge_name}")),
        LookupResult::ReassignedSubmapper {correct_mapper, correct_submapper } =>
            Err(format!("Mapper {number}, submapper {} has been reassigned to {correct_mapper}, {correct_submapper} . ROM: {cartridge_name}", sub_number.unwrap())),
    }
}

pub fn try_lookup_mapper(metadata: &ResolvedMetadata) -> LookupResult {
    use LookupResult::*;
    match (metadata.mapper_number, metadata.submapper_number) {
        // NROM
        (0, None) => m::mapper000::Mapper000.supported(),

        // MMC1 submappers
        (1, None) => UnspecifiedSubmapper,
        // Normal behavior
        (1, Some(0)) => match m::mapper001_0::Mapper001_0::new(metadata) {
            Ok(mapper) => mapper.supported(),
            Err(_) => ReassignedSubmapper { correct_mapper: 1, correct_submapper: 5 },
        }
        // SUROM, SOROM, SXROM
        (1, Some(1 | 2 | 4)) => ReassignedSubmapper { correct_mapper: 1, correct_submapper: 0 },
        (1, Some(3)) => ReassignedSubmapper { correct_mapper: 155, correct_submapper: 0 },
        // SEROM, SHROM, SH1ROM
        (1, Some(5)) => m::mapper001_5::Mapper001_5::default().supported(),
        // 2ME
        (1, Some(6)) => TodoSubmapper,

        // UxROM submappers
        (2, None | Some(0)) => UnspecifiedSubmapper,
        // No bus conflicts
        (2, Some(1)) => m::mapper002_1::MAPPER002_1.supported(),
        // Bus conflicts
        (2, Some(2)) => m::mapper002_2::MAPPER002_2.supported(),

        // CNROM submappers
        (3, None | Some(0)) => UnspecifiedSubmapper,
        // No bus conflicts
        (3, Some(1)) => m::mapper003_1::MAPPER003_1.supported(),
        // Bus conflicts
        (3, Some(2)) => m::mapper003_2::MAPPER003_2.supported(),

        // MMC3 submappers
        (4, None) => UnspecifiedSubmapper,
        // Sharp IRQs
        (4, Some(0)) => m::mapper004_0::mapper004_0().supported(),
        // MMC6
        (4, Some(1)) => m::mapper004_1::Mapper004_1::new().supported(),
        (4, Some(2)) => UnassignedSubmapper,
        // MC-ACC IRQs
        (4, Some(3)) => m::mapper004_3::mapper004_3().supported(),
        // NEC IRQs
        (4, Some(4)) => m::mapper004_4::mapper004_4().supported(),
        // T9552 scrambling chip
        (4, Some(5)) => TodoSubmapper,
        // Rev A IRQ doesn't have a submapper assigned to it, despite being incompatible.
        (4, Some(99)) => m::mapper004_rev_a::mapper004_rev_a().supported(),

        // MMC5
        (5, None) => m::mapper005::Mapper005::new().supported(),
        (6, _) => TodoMapper,

        // AxROM submappers
        (7, None | Some(0)) => UnspecifiedSubmapper,
        // No bus conflicts
        (7, Some(1)) => m::mapper007_1::MAPPER007_1.supported(),
        // Bus conflicts
        (7, Some(2)) => m::mapper007_2::MAPPER007_2.supported(),

        (8, None) => TodoMapper,
        // MMC2
        (9, None) => m::mapper009::Mapper009.supported(),
        // MMC4
        (10, None) => m::mapper010::Mapper010.supported(),
        // Color Dreams
        (11, None) => m::mapper011::Mapper011.supported(),
        (12, _) => TodoMapper,
        // NES CPROM
        (13, None) => m::mapper013::Mapper013.supported(),
        (14, _) => TodoMapper,
        // K-1029 and K-1030P
        (15, None) => m::mapper015::Mapper015.supported(),

        // Some Bandai FCG board submappers
        (16, None | Some(0)) => UnspecifiedSubmapper,
        (16, Some(1)) => ReassignedSubmapper { correct_mapper: 159, correct_submapper: 0 },
        (16, Some(2)) => ReassignedSubmapper { correct_mapper: 157, correct_submapper: 0 },
        (16, Some(3)) => ReassignedSubmapper { correct_mapper: 153, correct_submapper: 0 },
        // FCG-1
        (16, Some(4)) => m::mapper016_4::Mapper016_4::default().supported(),
        // LZ93D50
        (16, Some(5)) => m::mapper016_5::Mapper016_5::default().supported(),

        (17, _) => TodoMapper,
        // Jaleco SS 88006
        (18, None) => m::mapper018::Mapper018::default().supported(),
        // Namco 129 and Namco 163.
        // (Expansion Audio isn't supported yet, so all submappers are the same for now.)
        (19, None | Some(0)) => UnspecifiedSubmapper,
        // Duplicate of submapper 2.
        (19, Some(1)) => m::mapper019::Mapper019::new().supported(),
        (19, Some(2)) => m::mapper019::Mapper019::new().supported(),
        (19, Some(3)) => m::mapper019::Mapper019::new().supported(),
        (19, Some(4)) => m::mapper019::Mapper019::new().supported(),
        (19, Some(5)) => m::mapper019::Mapper019::new().supported(),
        // Only used for testing Famicom Disk System images, so it's not an actual iNES mapper.
        (20, _) => UnassignedMapper,

        // Some VRC4 submappers
        (21, None | Some(0)) => UnspecifiedSubmapper,
        // VRC4a
        (21, Some(1)) => m::mapper021_1::mapper021_1().supported(),
        // VRC4c
        (21, Some(2)) => m::mapper021_2::mapper021_2().supported(),

        // VRC2a
        (22, None) => m::mapper022::mapper022().supported(),

        // Some VRC2 and VRC4 submappers
        (23, None | Some(0)) => UnspecifiedSubmapper,
        // VRC4f
        (23, Some(1)) => m::mapper023_1::mapper023_1().supported(),
        // VRC4e
        (23, Some(2)) => m::mapper023_2::mapper023_2().supported(),
        // VRC2b
        (23, Some(3)) => m::mapper023_3::mapper023_3().supported(),

        (24, _) => TodoMapper,

        // Some VRC2 and VRC4 submappers
        (25, None | Some(0)) => UnspecifiedSubmapper,
        // VRC4b
        (25, Some(1)) => m::mapper025_1::mapper025_1().supported(),
        // VRC4d
        (25, Some(2)) => m::mapper025_2::mapper025_2().supported(),
        // VRC2c
        (25, Some(3)) => m::mapper025_3::mapper025_3().supported(),

        (26, None) => TodoMapper,
        // Duplicate of 23, most likely.
        (27, None) => m::mapper023_1::mapper023_1().supported(),
        // Action 53
        (28, None) => m::mapper028::Mapper028::new().supported(),
        // Homebrew. Sealie Computing - RET-CUFROM revD
        (29, None) => m::mapper029::Mapper029.supported(),
        (30, _) => TodoMapper,
        (31, _) => TodoMapper,

        // Irem G101 submappers
        (32, None) => UnspecifiedSubmapper,
        // Normal behavior
        (32, Some(0)) => m::mapper032::Mapper032.supported(),
        // One-screen mirroring, fixed PRG banks (only Major League)
        (32, Some(1)) => TodoSubmapper,

        // Taito's TC0190
        (33, None) => m::mapper033::Mapper033.supported(),

        // NINA-001 and BNROM submappers
        (34, None | Some(0)) => UnspecifiedSubmapper,
        // NINA-001
        (34, Some(1)) => m::mapper034_1::Mapper034_1.supported(),
        // BNROM
        (34, Some(2)) => m::mapper034_2::Mapper034_2.supported(),

        (35..=37, _) => TodoMapper,
        // Bit Corp.'s Crime Busters
        (38, None) => m::mapper038::Mapper038.supported(),
        // Duplicate of 241.
        (39, None) => m::mapper039::Mapper039::new().supported(),
        // NTDEC 2722 and NTDEC 2752 PCB and imitations
        (40, None) => m::mapper040::Mapper040::default().supported(),
        // Caltron 6-in-1
        (41, None) => m::mapper041::Mapper041::default().supported(),
        // FDS games hacked into cartridge form
        (42, None) => m::mapper042::Mapper042::new(metadata).supported(),
        // TONY-I and YS-612 (FDS games in cartridge form)
        (43, None) => m::mapper043::Mapper043::default().supported(),
        (44, _) => TodoMapper,
        (45, _) => TodoMapper,
        // Rumble Station
        (46, None) => m::mapper046::Mapper046::default().supported(),
        // Super Spike V'Ball + Nintendo World Cup
        (47, None) => m::mapper047::Mapper047::new().supported(),
        // Taito TC0690
        (48, None) => m::mapper048::Mapper048::new().supported(),
        (49, _) => TodoMapper,
        // N-32 conversion of Super Mario Bros. 2 (J). PCB code 761214.
        (50, None) => m::mapper050::Mapper050::default().supported(),
        (51..=54, _) => TodoMapper,
        // BTL-MARIO1-MALEE2
        (55, None) => m::mapper055::Mapper055.supported(),
        (56, _) => TodoMapper,
        (57, _) => TodoMapper,
        // NROM-/CNROM-based multicarts
        (58, None) => m::mapper058::Mapper058.supported(),
        (59, _) => TodoMapper,
        (60, _) => TodoMapper,
        // NTDEC 0324 PCB
        (61, None) => m::mapper061::Mapper061::new(metadata).supported(),
        // Super 700-in-1
        (62, None) => m::mapper062::Mapper062.supported(),

        // NTDEC's "Powerful 250-in-1" multicart and pirate equivalents
        (63, None) => UnspecifiedSubmapper,
        // TH2291-3 and CH-011
        (63, Some(0)) => m::mapper063_0::Mapper063_0.supported(),
        // 82AB
        (63, Some(1)) => m::mapper063_1::Mapper063_1.supported(),

        // RAMBO-1
        (64, None) => m::mapper064::Mapper064::new().supported(),
        // Irem's H3001
        (65, None) => m::mapper065::Mapper065::default().supported(),
        // GxROM (GNROM and MHROM)
        (66, None) => m::mapper066::Mapper066.supported(),
        // Sunsoft-3
        (67, None) => m::mapper067::Mapper067::default().supported(),
        (68, _) => TodoMapper,
        // Sunsoft FME-7
        (69, None) => m::mapper069::Mapper069::new().supported(),
        // Family Trainer and others
        (70, None) => m::mapper070::Mapper070.supported(),

        // Codemasters submappers
        (71, None) => UnspecifiedSubmapper,
        // Hardwired mirroring
        // FIXME: Implement specific submapper.
        (71, Some(0)) => m::mapper071::Mapper071.supported(),
        // Mapper-controlled mirroring (only Fire Hawk)
        // FIXME: Implement specific submapper.
        (71, Some(1)) => m::mapper071::Mapper071.supported(),

        (72, _) => TodoMapper,
        // VRC3
        (73, None) => m::mapper073::Mapper073::default().supported(),
        (74, _) => TodoMapper,
        // Konami VRC1
        (75, None) => m::mapper075::Mapper075::default().supported(),
        // NAMCOT-3446
        (76, None) => m::mapper076::Mapper076::new().supported(),
        // Irem (Napoleon Senki)
        (77, None) => m::mapper077::Mapper077.supported(),

        // Holy Diver and Uchuusen - Cosmo Carrier submappers
        (78, None | Some(0)) => UnspecifiedSubmapper,
        // Single-screen mirroring (only Cosmo Carrier)
        (78, Some(1)) => m::mapper078_1::Mapper078_1.supported(),
        // Mapper-controlled mirroring (only Holy Diver)
        (78, Some(3)) => m::mapper078_3::Mapper078_3.supported(),

        // NINA-03 and NINA-06
        (79, None) => m::mapper079::Mapper079.supported(),
        // Taito's X1-005
        (80, None) => m::mapper080::Mapper080.supported(),
        // Super Gun from NTDEC
        (81, None) => m::mapper081::Mapper081.supported(),
        // Taito X1-017
        (82, None) => m::mapper082::Mapper082.supported(),
        (83, _) => TodoMapper,
        (84, _) => UnassignedMapper,

        // Konami VRC7 submappers
        (85, None | Some(0)) => UnspecifiedSubmapper,
        // VRC7b - Tiny Toon Adventures
        (85, Some(1)) => m::mapper085_1::Mapper085_1::default().supported(),
        // VRC7a - Lagrange Point
        (85, Some(2)) => m::mapper085_2::Mapper085_2::default().supported(),

        // Jaleco JF-13
        (86, None) => m::mapper086::Mapper086.supported(),
        // Jaleco J87
        (87, None) => m::mapper087::Mapper087.supported(),
        // NAMCOT-3443
        (88, None) => m::mapper088::Mapper088::new(metadata).supported(),
        // Sunsoft (Tenka no Goikenban: Mito Koumon (J))
        (89, None) => m::mapper089::Mapper089.supported(),
        (90, _) => TodoMapper,

        // J.Y. Company clone boards and Super Fighter III submappers.
        (91, None) => UnspecifiedSubmapper,
        // J.Y. Company clone boards
        (91, Some(0)) => m::mapper091_0::Mapper091_0::default().supported(),
        // Super Fighter III
        (91, Some(1)) => m::mapper091_1::Mapper091_1::default().supported(),

        (92, _) => TodoMapper,
        // Sunsoft-2 IC on the Sunsoft-3R board
        (93, None) => m::mapper093::Mapper093.supported(),
        // HVC-UN1ROM
        (94, None) => m::mapper094::Mapper094.supported(),
        (95, _) => TodoMapper,
        (96, _) => TodoMapper,
        // Irem TAM-S1 (Kaiketsu Yanchamaru)
        (97, None) => m::mapper097::Mapper097.supported(),
        (98, _) => UnassignedMapper,
        (99, _) => TodoMapper,
        (100, _) => TodoMapper,
        // JF-10 misdump (only Urusei Yatsura - Lum no Wedding Bell)
        (101, None) => m::mapper101::MAPPER101.supported(),
        (102, _) => UnassignedMapper,
        (103..=106, _) => TodoMapper,
        // Magic Dragon
        (107, None) => m::mapper107::Mapper107.supported(),
        (108..=111, _) => TodoMapper,
        // Huang Di and San Guo Zhi - Qun Xiong Zheng Ba
        (112, None) => m::mapper112::Mapper112::new().supported(),
        // HES NTD-8
        (113, None) => m::mapper113::Mapper113.supported(),
        (114..=117, _) => TodoMapper,
        // TxSROM
        (118, None) => m::mapper118::Mapper118::new().supported(),
        // TQROM
        (119, None) => m::mapper119::Mapper119::new().supported(),
        (120, _) => TodoMapper,
        (121, _) => TodoMapper,
        // Duplicate
        (122, None) => m::mapper184::Mapper184.supported(),
        (123, _) => TodoMapper,
        (124, _) => TodoMapper,
        // Monty on the Run (Whirlwind Manu's FDS conversion)
        (125, None) => m::mapper125::Mapper125.supported(),
        (126..=132, _) => TodoMapper,
        // Sachen 3009
        (133, None) => m::mapper133::Mapper133.supported(),
        (134..=137, _) => TodoMapper,
        // Sachen 8259 B (UNL-Sachen-8259B)
        (138, None) => m::mapper138::MAPPER138.supported(),
        // Sachen 8259 C (UNL-Sachen-8259C)
        (139, None) => m::mapper139::MAPPER139.supported(),
        // Jaleco J-11 and J-14
        (140, None) => m::mapper140::Mapper140.supported(),
        // Sachen 8259 A TC-A003-72 (UNL-Sachen-8259A)
        (141, None) => m::mapper141::MAPPER141.supported(),
        // Kaiser KS202 (UNL-KS7032)
        (142, None) => m::mapper142::Mapper142::default().supported(),
        (143, _) => TodoMapper,
        (144, _) => TodoMapper,
        // SA-72007 (only Sidewinder)
        (145, None) => m::mapper145::Mapper145.supported(),
        // Duplicate of mapper 79, specifically for the Sachen 3015 board.
        (146, None) => m::mapper079::Mapper079.supported(),
        (147, _) => TodoMapper,
        // Sachen SA-008-A and Tengen 800008
        (148, None) => m::mapper148::Mapper148.supported(),
        // Sachen SA-0036 (Taiwan Mahjong 16)
        (149, None) => m::mapper149::Mapper149.supported(),
        (150, _) => TodoMapper,
        // Duplicate
        (151, None) => m::mapper075::Mapper075::default().supported(),
        // TAITO-74*161/161/32 and BANDAI-74*161/161/32
        (152, None) => m::mapper152::Mapper152.supported(),
        (153, _) => TodoMapper,
        // NAMCOT-3453 (only Devil Man)
        (154, None) => m::mapper154::Mapper154::new(metadata).supported(),
        (155..=158, _) => TodoMapper,
        // Almost a duplicate, but has different EEPROM behavior (not implemented yet).
        (159, None) => m::mapper016_4::Mapper016_4::default().supported(),
        (160, _) => TodoMapper,
        // Duplicate. Hanjuku Eiyuu (J).
        (161, None) => m::mapper001_0::Mapper001_0::new(metadata).unwrap().supported(),
        (162..=176, _) => TodoMapper,
        // Hengedianzi (恒格电子) two-screen mirroring
        (177, None) => m::mapper177::Mapper177.supported(),
        (178, _) => TodoMapper,
        (179, _) => TodoMapper,
        // UNROM 74HC08 (only Crazy Climber)
        (180, None) => m::mapper180::Mapper180.supported(),
        (181, _) => UnassignedMapper,
        (182, _) => TodoMapper,
        (183, _) => TodoMapper,
        // Sunsoft-1
        (184, None) => m::mapper184::Mapper184.supported(),
        // CNROM with CHR RAM disable
        (185, None) => UnspecifiedSubmapper,
        (185, Some(0)) => m::mapper185_0::Mapper185_0::default().supported(),
        (185, Some(4)) => m::mapper185_4::MAPPER185_4.supported(),
        (185, Some(5)) => m::mapper185_5::MAPPER185_5.supported(),
        (185, Some(6)) => m::mapper185_6::MAPPER185_6.supported(),
        (185, Some(7)) => m::mapper185_7::MAPPER185_7.supported(),

        // Used when running the BIOS of the Fukutake Study Box.
        (186, _) => UnassignedMapper,
        (187, _) => TodoMapper,
        (188, _) => TodoMapper,
        // TXC-PT8154
        (189, None) => m::mapper189::Mapper189::new().supported(),
        (190..=192, _) => TodoMapper,
        // NTDEC's TC-112
        (193, None) => m::mapper193::Mapper193.supported(),
        (194..=199, _) => TodoMapper,

        // NROM-128 multicarts
        (200, None) => UnspecifiedSubmapper,
        // More PRG/CHR banks
        (200, Some(0)) => m::mapper200_0::Mapper200_0.supported(),
        // Fewer PRG/CHR banks
        (200, Some(1)) => m::mapper200_1::Mapper200_1.supported(),

        // NROM-256 multicarts
        (201, None) => m::mapper201::Mapper201.supported(),
        // 150-in-1 pirate cart
        (202, None) => m::mapper202::Mapper202.supported(),
        // 35-in-1
        (203, None) => m::mapper203::Mapper203.supported(),
        (204, _) => TodoMapper,
        (205, _) => TodoMapper,
        // DxROM, Tengen MIMIC-1, Namcot 118
        (206, None) => UnspecifiedSubmapper,
        // Normal PRG banking
        (206, Some(0)) => m::mapper206::Mapper206::new().supported(),
        // Fixed 32KiB PRG bank
        (206, Some(1)) => TodoSubmapper,
        // Taito's X1-005 (alternate name table mirrorings)
        (207, None) => m::mapper207::Mapper207::new().supported(),
        (208, _) => TodoMapper,
        (209, _) => TodoMapper,

        // Namco 175 and 340 submappers
        (210, None | Some(0)) => UnspecifiedSubmapper,
        // Namco 175
        (210, Some(1)) => m::mapper210_1::Mapper210_1.supported(),
        // Namco 340
        (210, Some(2)) => m::mapper210_2::Mapper210_2.supported(),

        (211, _) => TodoMapper,
        (212, _) => TodoMapper,
        // Duplicate
        (213, None) => m::mapper058::Mapper058.supported(),
        (214..=231, _) => TodoMapper,

        // Quattro submappers
        (232, None) => UnspecifiedSubmapper,
        // Normal behavior
        (232, Some(0)) => m::mapper232::Mapper232.supported(),
        // Aladdin Deck Enhancer
        (232, Some(1)) => TodoSubmapper,

        (233, _) => TodoMapper,
        // Maxi 15 multicart
        (234, None) => m::mapper234::Mapper234::default().supported(),
        (235..=238, _) => TodoMapper,
        (239, _) => UnassignedMapper,
        (240, None) => m::mapper240::Mapper240.supported(),
        // Hengedianzi (恒格电子) hard-wired mirroring, and mapper hacks (m 164, 178, 227)
        (241, None) => m::mapper241::Mapper241.supported(),
        (242..=246, _) => TodoMapper,
        (247, _) => UnassignedMapper,
        (248..=255, _) => TodoMapper,

        (464..=466, _) => UnassignedMapper,
        (475, _) => UnassignedMapper,
        (477..=478, _) => UnassignedMapper,
        (480, _) => UnassignedMapper,
        (482..=486, _) => UnassignedMapper,
        (488..=492, _) => UnassignedMapper,
        (494, _) => UnassignedMapper,
        (496, _) => UnassignedMapper,
        (498..=511, _) => UnassignedMapper,
        (559..=560, _) => UnassignedMapper,
        (563..=681, _) => UnassignedMapper,
        (683..=767, _) => UnassignedMapper,
        (256..=767, _) => TodoMapper,
        // Assigning these numbers would require further extension of the iNES/NES2.0 format.
        (768..=65535, _) => UnassignedMapper,
        (_, Some(_)) => UnassignedSubmapper,
    }
}
