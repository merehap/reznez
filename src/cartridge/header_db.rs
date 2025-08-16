#![allow(clippy::unreadable_literal)]
#![allow(clippy::zero_prefixed_literal)]

use std::collections::BTreeMap;

use log::info;

use crate::cartridge::{cartridge::Cartridge, cartridge_header::{CartridgeHeader, CartridgeHeaderBuilder}};

// Submapper numbers for ROMs that aren't in the NES Header DB (mostly test ROMs).
const MISSING_ROM_SUBMAPPER_NUMBERS: &[(u32, u32, u16, u8)] = &[
    // ppu_read_buffer/test_ppu_read_buffer.nes
    (1731018083, 3047829474, 3, 1),
    // cpu_dummy_reads.nes
    (3745301915, 3498657192, 3, 1),
    // read_joy3/thorough_test.nes
    (4081207045, 0068811153, 3, 1),
    // read_joy3/count_errors.nes
    (2480941058, 571989249, 3, 1),
    // read_joy3/count_errors_fast.nes
    (401642813, 392036799, 3, 1),
    // read_joy3/test_buttons.nes
    (3991554801, 3461020892, 3, 1),

    // 2_test/2_test_0.nes
    (2392242790, 3901840109, 2, 1),
    // 2_test/2_test_1.nes
    (0922356069, 3901840109, 2, 1),
    // 2_test/2_test_2.nes
    (0624876065, 3901840109, 2, 2),
    // 3_test/3_test_0.nes
    (2333203173, 3631928862, 3, 1),
    // 3_test/3_test_1.nes
    (2768833268, 3631928862, 3, 1),
    // 3_test/3_test_2.nes
    (3609230023, 3631928862, 3, 2),
    // 7_test/7_test_0.nes
    (1196595180, 1718968027, 7, 1),
    // 7_test/7_test_1.nes
    (4282262767, 1718968027, 7, 1),
    // 7_test/7_test_2.nes
    (3975870379, 1718968027, 7, 2),

    // bntest/bntest_aorom.nes
    (0271080456, 3417935230, 7, 1),
    // bntest/bntest_h.nes
    (2219365086, 3417935230, 34, 2),
    // bntest/bntest_v.nes
    (0180120094, 3417935230, 34, 2),

    // holydiverbatman/M2_P128K_V.nes
    (3834536932, 3161185153, 2, 1),
    // holydiverbatman/M3_P32K_C32K_H.nes
    (0403828385, 3300419450, 3, 1),
    // holydiverbatman/M7_P128K.nes
    (0656988303, 3161185153, 7, 1),
    // holydiverbatman/M34_P128K_H.nes
    (0819388560, 3161185153, 34, 1),
    // holydiverbatman/M78.3_P128K_C64K.nes
    (2751966167, 3161185153, 78, 3),

    // instr_misc.nes
    (1160500958, 3165947151, 1, 0),
    // instr_timing.nes
    (2178783783, 1558157791, 1, 0),
    // all_instrs.nes
    (2755660131, 036867474, 1, 0),

    // mmc3_test/1-clocking.nes
    (0840815252, 4211641636, 4, 0),
    // mmc3_test/2-details.nes
    (1461067563, 4091089803, 4, 0),
    // mmc3_test/3-A12_clocking.nes
    (3522479107, 1525174205, 4, 0),
    // mmc3_test/4-scanline_timing.nes
    (3213815090, 2170500801, 4, 0),
    // mmc3_test/5-MMC3.nes
    (3729061091, 3319686763, 4, 0),
    // mmc3_test/6-MMC6.nes
    (2669308141, 2914571485, 4, 1),
    // mmc3_irq_tests/1.Clocking.nes
    (1372524753, 3210513450, 4, 0),
    // mmc3_irq_tests/2.Details.nes
    (2314296442, 1734525057, 4, 0),
    // mmc3_irq_tests/3.A12_clocking.nes
    (0515143863, 4029146188, 4, 0),
    // mmc3_irq_tests/4.Scanline_timing.nes
    (0440252636, 4105053223, 4, 0),
    // mmc3_irq_tests/5.MMC3_rev_A.nes (no submapper number has been officially assigned)
    (0495013157, 4078096862, 4, 99),
    // mmc3_irq_tests/6.MMC3_rev_B.nes
    (2174952589, 1865463926, 4, 0),

    // shxing1.nes
    (1996547387, 408545878, 7, 1),
    // shxing2.nes
    (675601020, 1193137425, 7, 1),
    // shxdma.nes
    (4274158895, 2442883650, 7, 1),

    // Crystalis (no submapper number has been officially assigned)
    (656187357, 1661724784, 4, 99),

    // Lagrange Point
    (2905171667, 869154800, 85, 2),

    // Dragon Ball Z - Kyoushuu! Saiya Jin (J)
    (3020245841, 1651092634, 16, 5)
];

pub struct HeaderDb {
    data_by_crc32: BTreeMap<u32, CartridgeHeader>,
    prg_rom_by_crc32: BTreeMap<u32, CartridgeHeader>,
    missing_data_submapper_numbers: BTreeMap<u32, (u16, u8)>,
    missing_prg_rom_submapper_numbers: BTreeMap<u32, (u16, u8)>,
}

impl HeaderDb {
    pub fn load() -> HeaderDb {
        let text = include_str!("../../nes20db.xml");
        let doc = roxmltree::Document::parse(text).unwrap();
        let games = doc.root().descendants().filter(|n| n.tag_name().name() == "game");

        let missing_data_submapper_numbers: BTreeMap<u32, (u16, u8)> =
            MISSING_ROM_SUBMAPPER_NUMBERS.iter().map(|(k, _, m, s)| (*k, (*m, *s))).collect();
        let missing_prg_rom_submapper_numbers: BTreeMap<u32, (u16, u8)> =
            MISSING_ROM_SUBMAPPER_NUMBERS.iter().map(|(_, k, m, s)| (*k, (*m, *s))).collect();

        let mut header_db = HeaderDb {
            data_by_crc32: BTreeMap::new(),
            prg_rom_by_crc32: BTreeMap::new(),
            missing_data_submapper_numbers,
            missing_prg_rom_submapper_numbers,
        };

        for game in games {
            let data_hash = read_attribute(game, "rom", "crc32").unwrap();
            let full_hash = u32::from_str_radix(data_hash, 16).unwrap();
            let prg_rom_hash = read_attribute(game, "prgrom", "crc32").unwrap();
            let prg_rom_hash = u32::from_str_radix(prg_rom_hash, 16).unwrap();
            let mut header_builder = CartridgeHeaderBuilder::new();
            header_builder
                .full_hash(full_hash)
                .prg_rom_hash(prg_rom_hash)
                .prg_rom_size(read_attribute(game, "prgrom", "size").unwrap().parse().unwrap())
                .mapper_number(read_attribute(game, "pcb", "mapper").unwrap().parse().unwrap())
                .submapper_number(read_attribute(game, "pcb", "submapper").unwrap().parse().unwrap());

            read_attribute(game, "prgram", "size").inspect(|s| { header_builder.prg_work_ram_size(s.parse().unwrap()); });
            read_attribute(game, "prgnvram", "size").inspect(|s| { header_builder.prg_save_ram_size(s.parse().unwrap()); });
            read_attribute(game, "chrrom", "size").inspect(|s| { header_builder.chr_rom_size(s.parse().unwrap()); });
            read_attribute(game, "chrram", "size").inspect(|s| { header_builder.chr_work_ram_size(s.parse().unwrap()); });
            read_attribute(game, "chrnvram", "size").inspect(|s| { header_builder.chr_save_ram_size(s.parse().unwrap()); });

            let header = header_builder.build();
            header_db.data_by_crc32.insert(full_hash, header.clone());
            header_db.prg_rom_by_crc32.insert(prg_rom_hash, header);
        }

        header_db
    }

    pub fn header_from_db(
        &self,
        cartridge: &Cartridge,
        full_hash: u32,
        prg_hash: u32,
        mapper_number: u16,
        submapper_number: Option<u8>,
    ) -> Option<CartridgeHeader> {

        let mut override_submapper_number = None;
        if let Some((number, sub_number)) = self.missing_data_submapper_numbers.get(&full_hash).copied()
                && number == cartridge.mapper_number() {
            info!("Using override submapper for this ROM. Full hash: {full_hash} , PRG hash: {prg_hash}");
            override_submapper_number = Some(sub_number);
        } else if let Some((number, sub_number)) = self.missing_prg_rom_submapper_numbers.get(&prg_hash).copied()
                && number == cartridge.mapper_number() {
            info!("Using override submapper for this ROM. Full hash: {full_hash} , PRG hash: {prg_hash}");
            override_submapper_number = Some(sub_number);
        }

        let mut result = self.data_by_crc32.get(&full_hash).cloned();
        if let Some(ref mut header) = result {
            if let Some(override_submapper_number) = override_submapper_number {
                header.set_submapper_number(override_submapper_number);
                return Some(header.clone());
            } else {
                return result;
            }
        }

        if result.is_some() {
            return result;
        }

        let mut result = self.prg_rom_by_crc32.get(&prg_hash).cloned();
        if result.is_none() {
            if let Some(submapper_number) = submapper_number {
                info!("ROM not found in DB. ({full_hash}, {prg_hash}, {mapper_number}, {submapper_number})");
            } else {
                info!("ROM not found in DB. ({full_hash}, {prg_hash}, {mapper_number}, ???)");
            }
        }

        if let Some(ref mut header) = result && let Some(override_submapper_number) = override_submapper_number {
            header.set_submapper_number(override_submapper_number);
            Some(header.clone())
        } else {
            result
        }
    }

    pub fn missing_submapper_number(&self, data_hash: u32, prg_hash: u32) -> Option<(u16, u8, u32, u32)> {
        if let Some((number, sub_number)) = self.missing_data_submapper_numbers.get(&data_hash).copied() {
            Some((number, sub_number, data_hash, prg_hash))
        } else if let Some((number, sub_number)) = self.missing_prg_rom_submapper_numbers.get(&prg_hash).copied() {
            Some((number, sub_number, data_hash, prg_hash))
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