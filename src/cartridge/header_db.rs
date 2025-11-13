#![allow(clippy::unreadable_literal)]
#![allow(clippy::zero_prefixed_literal)]

use std::collections::BTreeMap;

use log::info;
use num_traits::FromPrimitive;

use crate::{cartridge::cartridge_metadata::{CartridgeMetadata, CartridgeMetadataBuilder}, mapper::NameTableMirroring};

const OVERRIDE_SUBMAPPER_NUMBERS: &[(u32, u32, u16, u8)] = &[
    // Crystalis (no submapper number has been officially assigned for MMC3 with Sharp Rev A IRQ)
    (0x271C9FDD, 0x630BE870, 4, 99),
];

// Submapper numbers for ROMs that aren't in the NES Header DB (mostly test ROMs).
const MISSING_ROM_SUBMAPPER_NUMBERS: &[(u32, u32, u16, u8)] = &[
    // ppu_read_buffer/test_ppu_read_buffer.nes
    (0x672D3D63, 0xB5AA2FE2, 3, 1),
    // cpu_dummy_reads.nes
    (0xDF3CC59B, 0xD08945A8, 3, 1),
    // read_joy3/thorough_test.nes
    (0xF3424705, 0x0419F991, 3, 1),
    // read_joy3/count_errors.nes
    (0x93E02802, 0x2217DD01, 3, 1),
    // read_joy3/count_errors_fast.nes
    (0x17F0953D, 0x175E01BF, 3, 1),
    // read_joy3/test_buttons.nes
    (0xEDEA4AF1, 0xCE4AFCDC, 3, 1),

    // 2_test/2_test_0.nes
    (0x8E96BA66, 0xE8915AED, 2, 1),
    // 2_test/2_test_1.nes
    (0x36FA0965, 0xE8915AED, 2, 1),
    // 2_test/2_test_2.nes
    (0x253EDA21, 0xE8915AED, 2, 2),
    // 3_test/3_test_0.nes
    (0x8B11DAE5, 0xD87AD61E, 3, 1),
    // 3_test/3_test_1.nes
    (0xA5090AF4, 0xD87AD61E, 3, 1),
    // 3_test/3_test_2.nes
    (0xD7207AC7, 0xD87AD61E, 3, 2),
    // 7_test/7_test_0.nes
    (0x475297EC, 0x66755EDB, 7, 1),
    // 7_test/7_test_1.nes
    (0xFF3E24EF, 0x66755EDB, 7, 1),
    // 7_test/7_test_2.nes
    (0xECFAF7AB, 0x66755EDB, 7, 2),

    // bntest/bntest_aorom.nes
    (0x10285C08, 0xCBB98D7E, 7, 1),
    // bntest/bntest_h.nes
    (0x8448D2DE, 0xCBB98D7E, 34, 2),
    // bntest/bntest_v.nes
    (0x0ABC6A1E, 0xCBB98D7E, 34, 2),

    // holydiverbatman/M2_P128K_V.nes
    (0xE48E63E4, 0xBC6BDB81, 2, 1),
    // holydiverbatman/M3_P32K_C32K_H.nes
    (0x1811EEA1, 0xC4B8677A, 3, 1),
    // holydiverbatman/M7_P128K.nes
    (0x2728D88F, 0xBC6BDB81, 7, 1),
    // holydiverbatman/M34_P128K_H.nes
    (0x30D6E090, 0xBC6BDB81, 34, 1),
    // holydiverbatman/M78.3_P128K_C64K.nes
    (0xA407ABD7, 0xBC6BDB81, 78, 3),

    // instr_misc.nes
    (0x452BD6DE, 0xBCB4850F, 1, 0),
    // instr_timing.nes
    (0x81DD9A27, 0x5CDF99DF, 1, 0),
    // all_instrs.nes
    (0xA4400963, 0x2328D92, 1, 0),

    // mmc3_test/1-clocking.nes
    (0x321DD294, 0xFB088D24, 4, 0),
    // mmc3_test/2-details.nes
    (0x57161F2B, 0xF3D9138B, 4, 0),
    // mmc3_test/3-A12_clocking.nes
    (0xD1F4C403, 0x5AE84FBD, 4, 0),
    // mmc3_test/4-scanline_timing.nes
    (0xBF8EED32, 0x815F36C1, 4, 0),
    // mmc3_test/5-MMC3.nes
    (0xDE44F4E3, 0xC5DE666B, 4, 0),
    // mmc3_test/6-MMC6.nes
    (0x9F1A68ED, 0xADB8D4DD, 4, 1),
    // mmc3_irq_tests/1.Clocking.nes
    (0x51CF10D1, 0xBF5C8C2A, 4, 0),
    // mmc3_irq_tests/2.Details.nes
    (0x89F15C7A, 0x6762C081, 4, 0),
    // mmc3_irq_tests/3.A12_clocking.nes
    (0x1EB478B7, 0xF027E44C, 4, 0),
    // mmc3_irq_tests/4.Scanline_timing.nes
    (0x1A3DB8DC, 0xF4AE2427, 4, 0),
    // mmc3_irq_tests/5.MMC3_rev_A.nes (no submapper number has been officially assigned)
    (0x1D814D25, 0xF312D1DE, 4, 99),
    // mmc3_irq_tests/6.MMC3_rev_B.nes
    (0x81A3248D, 0x6F30B876, 4, 0),

    // shxing1.nes
    (0x7700E53B, 0x1859EA56, 7, 1),
    // shxing2.nes
    (0x2844DA7C, 0x471DD511, 7, 1),
    // shxdma.nes
    (0xFEC27D2F, 0x919B7242, 7, 1),

    // Lagrange Point
    (0xAD2966D3, 0x33CE3FF0, 85, 2),

    // Dragon Ball Z - Kyoushuu! Saiya Jin (J)
    (0xB4054B51, 0x6269AC9A, 10, 5),

    // Commando (U) [b1][T+Bra_BRGames]
    // Not necessarily the correct submapper number.
    (0xF458222C, 0xB84451FB, 7, 1),

    // Dragon Ball Z - Kyoushuu! Saiya Jin (J)
    // Not necessarily the correct submapper number.
    (0xB4054B51, 0x6269AC9A, 16, 4),
];

pub struct HeaderDb {
    metadata_by_full_crc32: BTreeMap<u32, CartridgeMetadata>,
    metadata_by_prg_rom_crc32: BTreeMap<u32, CartridgeMetadata>,

    missing_submapper_numbers_by_full_hash: BTreeMap<u32, (u16, CartridgeMetadata)>,
    missing_submapper_numbers_by_prg_rom_hash: BTreeMap<u32, (u16, CartridgeMetadata)>,

    override_submapper_numbers_by_full_hash: BTreeMap<u32, (u16, CartridgeMetadata)>,
    override_submapper_numbers_by_prg_rom_hash: BTreeMap<u32, (u16, CartridgeMetadata)>,
}

impl HeaderDb {
    pub fn load() -> HeaderDb {
        let text = include_str!("../../nes20db.xml");
        let doc = roxmltree::Document::parse(text).unwrap();
        let games = doc.root().descendants().filter(|n| n.tag_name().name() == "game");

        let mut metadata_by_full_crc32 = BTreeMap::new();
        let mut metadata_by_prg_rom_crc32 = BTreeMap::new();
        for game in games {
            let full_hash = read_attribute(game, "rom", "crc32").unwrap();
            let full_hash = u32::from_str_radix(full_hash, 16).unwrap();

            let prg_rom_hash = read_attribute(game, "prgrom", "crc32").unwrap();
            let prg_rom_hash = u32::from_str_radix(prg_rom_hash, 16).unwrap();

            let mut header_builder = CartridgeMetadataBuilder::new();
            header_builder
                .full_hash(full_hash)
                .prg_rom_hash(prg_rom_hash)
                .mapper_and_submapper_number(
                    read_attribute(game, "pcb", "mapper").unwrap().parse().unwrap(),
                    read_attribute(game, "pcb", "submapper").unwrap().parse().ok()
                )
                .has_persistent_memory(read_attribute(game, "pcb", "battery").unwrap() == "1")
                .prg_rom_size(read_attribute(game, "prgrom", "size").unwrap().parse().unwrap())
                .prg_work_ram_size(read_attribute(game, "prgram", "size").map_or(0, |s| s.parse().unwrap()))
                .prg_save_ram_size(read_attribute(game, "prgnvram", "size").map_or(0, |s| s.parse().unwrap()))
                .chr_rom_size(read_attribute(game, "chrrom", "size").map_or(0, |s| s.parse().unwrap()))
                .chr_work_ram_size(read_attribute(game, "chrram", "size").map_or(0, |s| s.parse().unwrap()))
                .chr_save_ram_size(read_attribute(game, "chrnvram", "size").map_or(0, |s| s.parse().unwrap()))
                .console_type(read_attribute(game, "console", "type").map(|c| FromPrimitive::from_u8(c.parse().unwrap()).unwrap()).unwrap())
                .timing_mode(read_attribute(game, "console", "region").map(|t| FromPrimitive::from_u8(t.parse().unwrap()).unwrap()).unwrap())
                .default_expansion_device(read_attribute(game, "expansion", "type").map(|d| FromPrimitive::from_u8(d.parse().unwrap()).unwrap()).unwrap());

            if let Some(chr_rom_hash) = read_attribute(game, "chrrom", "crc32") {
                let chr_rom_hash = u32::from_str_radix(chr_rom_hash, 16).unwrap();
                header_builder.chr_rom_hash(chr_rom_hash);
            }

            if let Some(miscellaneous_rom_count) = read_attribute(game, "miscrom", "number") {
                header_builder.miscellaneous_rom_count(miscellaneous_rom_count.parse().unwrap());
            }

            if let (Some(hardware_type), Some(ppu_type)) = (read_attribute(game, "vs", "hardware"), read_attribute(game, "vs", "ppu")) {
                header_builder
                    .vs_hardware_type(FromPrimitive::from_u8(hardware_type.parse().unwrap()).unwrap())
                    .vs_ppu_type(FromPrimitive::from_u8(ppu_type.parse().unwrap()).unwrap());
            }

            if let Some(name_table_mirroring) = NameTableMirroring::from_short_string(read_attribute(game, "pcb", "mirroring").unwrap()).unwrap() {
                header_builder.name_table_mirroring(name_table_mirroring);
            }

            let header = header_builder.build();
            metadata_by_full_crc32.insert(full_hash, header.clone());
            metadata_by_prg_rom_crc32.insert(prg_rom_hash, header);
        }

        let missing_submapper_numbers_by_full_hash: BTreeMap<u32, (u16, CartridgeMetadata)> =
            MISSING_ROM_SUBMAPPER_NUMBERS.iter().map(|(k, _, m, s)| {
                assert!(!metadata_by_full_crc32.contains_key(k),
                    "ROM must NOT be in both header DB and DB extension. Full hash: {k}");
                let header = CartridgeMetadataBuilder::new().mapper_and_submapper_number(*m, Some(*s)).build();
                (*k, (*m, header))
            }).collect();
        let missing_submapper_numbers_by_prg_rom_hash: BTreeMap<u32, (u16, CartridgeMetadata)> =
            MISSING_ROM_SUBMAPPER_NUMBERS.iter().map(|(_, k, m, s)| {
                assert!(!metadata_by_prg_rom_crc32.contains_key(k),
                    "ROM must NOT be in both header DB and DB extension. PRG ROM hash: {k}");
                let header = CartridgeMetadataBuilder::new().mapper_and_submapper_number(*m, Some(*s)).build();
                (*k, (*m, header))
            }).collect();

        let override_submapper_numbers_by_full_hash: BTreeMap<u32, (u16, CartridgeMetadata)> =
            OVERRIDE_SUBMAPPER_NUMBERS.iter().map(|(k, _, m, s)| {
                let header = CartridgeMetadataBuilder::new().mapper_and_submapper_number(*m, Some(*s)).build();
                (*k, (*m, header))
            }).collect();
        let override_submapper_numbers_by_prg_rom_hash: BTreeMap<u32, (u16, CartridgeMetadata)> =
            OVERRIDE_SUBMAPPER_NUMBERS.iter().map(|(_, k, m, s)| {
                let header = CartridgeMetadataBuilder::new().mapper_and_submapper_number(*m, Some(*s)).build();
                (*k, (*m, header))
            }).collect();

        HeaderDb {
            metadata_by_full_crc32,
            metadata_by_prg_rom_crc32,
            missing_submapper_numbers_by_full_hash,
            missing_submapper_numbers_by_prg_rom_hash,
            override_submapper_numbers_by_full_hash,
            override_submapper_numbers_by_prg_rom_hash,
        }
    }

    pub fn header_from_db(
        &self,
        full_hash: u32,
        prg_hash: u32,
        mapper_number: u16,
        submapper_number: Option<u8>,
    ) -> Option<CartridgeMetadata> {

        let result = self.metadata_by_full_crc32.get(&full_hash).cloned();
        if result.is_some() {
            return result;
        }

        let result = self.metadata_by_prg_rom_crc32.get(&prg_hash).cloned();
        if result.is_none() {
            if let Some(submapper_number) = submapper_number {
                info!("ROM not found in DB. (0x{full_hash:X}, 0x{prg_hash:X}, {mapper_number}, {submapper_number})");
            } else {
                info!("ROM not found in DB. (0x{full_hash:X}, 0x{prg_hash:X}, {mapper_number}, ???)");
            }
        }

        result
    }

    pub fn override_submapper_number(&self, data_hash: u32, prg_hash: u32) -> Option<(u16, u8, u32, u32)> {
        if let Some((number, header)) = self.override_submapper_numbers_by_full_hash.get(&data_hash).cloned() {
            Some((number, header.submapper_number().unwrap(), data_hash, prg_hash))
        } else if let Some((number, header)) = self.override_submapper_numbers_by_prg_rom_hash.get(&prg_hash).cloned() {
            Some((number, header.submapper_number().unwrap(), data_hash, prg_hash))
        } else {
            None
        }
    }

    pub fn missing_submapper_number(&self, data_hash: u32, prg_hash: u32) -> Option<(u16, u8, u32, u32)> {
        if let Some((number, header)) = self.missing_submapper_numbers_by_full_hash.get(&data_hash).cloned() {
            Some((number, header.submapper_number().unwrap(), data_hash, prg_hash))
        } else if let Some((number, header)) = self.missing_submapper_numbers_by_prg_rom_hash.get(&prg_hash).cloned() {
            Some((number, header.submapper_number().unwrap(), data_hash, prg_hash))
        } else {
            None
        }
    }
}

fn read_attribute<'a>(node: roxmltree::Node<'a, 'a>, child_name: &str, attribute_name: &str) -> Option<&'a str> {
    Some(node.children()
        .find(|n| n.tag_name().name() == child_name)?
        .attribute(attribute_name)
        .unwrap()
    )
}
