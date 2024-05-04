use std::collections::BTreeMap;

use log::info;

// Submapper numbers for ROMs that aren't in the NES Header DB (mostly test ROMs).
const MISSING_ROM_SUBMAPPER_NUMBERS: &'static [(u32, u32, u16, u8)] = &[
    // ppu_read_buffer/test_ppu_read_buffer.nes
    (1731018083, 3047829474, 3, 1),
    // cpu_dummy_reads.nes
    (3745301915, 3498657192, 3, 1),
    // read_joy3/thorough_test.nes
    (4081207045, 0068811153, 3, 1),
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
    // 34_test/34_test_1.nes
    (2768347885, 0852594764, 34, 1),
    // 34_test/34_test_2.nes
    (0906378520, 3170842144, 34, 2),
    // serom/serom.nes
    (2444067993, 3660366606, 1, 5),

    // holydiverbatman/M2_P128K_V.nes
    (3834536932, 3161185153, 2, 1),
    // holydiverbatman/M3_P32K_C32K_H.nes
    (0403828385, 3300419450, 3, 1),
    // holydiverbatman/M7_P128K.nes
    (0656988303, 3161185153, 7, 1),

    // mmc3_test/6-MMC6.nes
    (2669308141, 2914571485, 4, 1),

    // mmc3_irq_tests/5.MMC3_rev_A.nes (no submapper number has been officially assigned)
    (495013157, 4078096862, 4, 99),
    // Crystalis (no submapper number has been officially assigned)
    (656187357, 1661724784, 4, 99),
];

pub struct HeaderDb {
    data_by_crc32: BTreeMap<u32, Header>,
    prg_rom_by_crc32: BTreeMap<u32, Header>,
    missing_data_submapper_numbers: BTreeMap<u32, (u16, u8)>,
    missing_prg_rom_submapper_numbers: BTreeMap<u32, (u16, u8)>,
}

impl HeaderDb {
    pub fn load() -> HeaderDb {
        let text = include_str!("../../nes20db.xml");
        let doc = roxmltree::Document::parse(text).unwrap();
        let games = doc.root().descendants().filter(|n| n.tag_name().name() == "game");

        let missing_data_submapper_numbers: BTreeMap<u32, (u16, u8)> =
            BTreeMap::from_iter(MISSING_ROM_SUBMAPPER_NUMBERS.iter().map(|(k, _, m, s)| (*k, (*m, *s))));
        let missing_prg_rom_submapper_numbers: BTreeMap<u32, (u16, u8)> =
            BTreeMap::from_iter(MISSING_ROM_SUBMAPPER_NUMBERS.iter().map(|(_, k, m, s)| (*k, (*m, *s))));

        let mut header_db = HeaderDb {
            data_by_crc32: BTreeMap::new(),
            prg_rom_by_crc32: BTreeMap::new(),
            missing_data_submapper_numbers,
            missing_prg_rom_submapper_numbers,
        };

        for game in games {
            let header = Header {
                prg_rom_size: read_attribute(game, "prgrom", "size").unwrap().parse().unwrap(),
                prg_ram_size: read_attribute(game, "prgram", "size").unwrap_or("0").parse().unwrap(),
                chr_rom_size: read_attribute(game, "chrrom", "size").unwrap_or("0").parse().unwrap(),
                chr_ram_size: read_attribute(game, "chrram", "size").unwrap_or("0").parse().unwrap(),
                mapper_number: read_attribute(game, "pcb", "mapper").unwrap().parse().unwrap(),
                submapper_number: read_attribute(game, "pcb", "submapper").unwrap().parse().unwrap(),
            };

            let rom_crc32 = read_attribute(game, "rom", "crc32").unwrap();
            header_db.data_by_crc32.insert(u32::from_str_radix(rom_crc32, 16).unwrap(), header);
            let prg_rom_crc32 = read_attribute(game, "prgrom", "crc32").unwrap();
            header_db.prg_rom_by_crc32.insert(u32::from_str_radix(prg_rom_crc32, 16).unwrap(), header);
        }

        header_db
    }

    pub fn header_from_data(&self, data: &[u8]) -> Option<Header> {
        let hash = crc32fast::hash(data);
        let result = self.data_by_crc32.get(&crc32fast::hash(data)).copied();
        if result.is_none() {
            info!("ROM with full file hash {hash} not found in DB.");
        }

        result
    }

    pub fn header_from_prg_rom(&self, prg_rom: &[u8]) -> Option<Header> {
        let hash = crc32fast::hash(prg_rom);
        let result = self.prg_rom_by_crc32.get(&hash).copied();
        if result.is_none() {
            info!("ROM with PRG hash {hash} not found in DB.");
        }

        result
    }

    pub fn missing_submapper_number(&self, data: &[u8], prg_rom: &[u8]) -> Option<(u16, u8, u32, u32)> {
        let data_hash = crc32fast::hash(data);
        let prg_hash = crc32fast::hash(prg_rom);
        if let Some((mapper_number, submapper_number)) = self.missing_data_submapper_numbers.get(&data_hash).copied() {
            Some((mapper_number, submapper_number, data_hash, prg_hash))
        } else if let Some((mapper_number, submapper_number)) = self.missing_prg_rom_submapper_numbers.get(&prg_hash).copied() {
            Some((mapper_number, submapper_number, data_hash, prg_hash))
        } else {
            None
        }
    }
}

fn read_attribute<'a>(node: roxmltree::Node<'a, 'a>, child_name: &str, attribute_name: &str) -> Option<&'a str> {
    Some(node.children()
        .filter(|n| n.tag_name().name() == child_name)
        .next()?
        .attribute(attribute_name)
        .unwrap()
    )
}

#[derive(Clone, Copy, Debug)]
pub struct Header {
    pub prg_rom_size: u32,
    pub prg_ram_size: u32,
    pub chr_rom_size: u32,
    pub chr_ram_size: u32,
    pub mapper_number: u16,
    pub submapper_number: u8,
}
