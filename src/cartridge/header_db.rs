use std::collections::BTreeMap;

// Submapper numbers for ROMs that aren't in the NES Header DB (mostly test ROMs).
const MISSING_ROM_SUBMAPPER_NUMBERS: &'static [(u32, u8)] = &[
    // mmc3_test/6-MMC6.nes
    (2914571485, 1)
];

pub struct HeaderDb {
    data_by_crc32: BTreeMap<u32, Header>,
    prg_rom_by_crc32: BTreeMap<u32, Header>,
    missing_rom_submapper_numbers: BTreeMap<u32, u8>,
}

impl HeaderDb {
    pub fn load() -> HeaderDb {
        let text = include_str!("../../nes20db.xml");
        let doc = roxmltree::Document::parse(text).unwrap();
        let games = doc.root().descendants().filter(|n| n.tag_name().name() == "game");

        let mut header_db = HeaderDb {
            data_by_crc32: BTreeMap::new(),
            prg_rom_by_crc32: BTreeMap::new(),
            missing_rom_submapper_numbers: BTreeMap::from_iter(MISSING_ROM_SUBMAPPER_NUMBERS.iter().cloned()),
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
        self.data_by_crc32.get(&crc32fast::hash(data)).copied()
    }

    pub fn header_from_prg_rom(&self, prg_rom: &[u8]) -> Option<Header> {
        self.prg_rom_by_crc32.get(&crc32fast::hash(prg_rom)).copied()
    }

    pub fn missing_rom_submapper_number(&self, prg_rom: &[u8]) -> Result<u8, String> {
        let hash = crc32fast::hash(prg_rom);
        if let Some(submapper_number) = self.missing_rom_submapper_numbers.get(&hash).copied() {
            Ok(submapper_number)
        } else {
            Err(format!("Missing ROM with CRC32 of {hash} needs a fallback entry."))
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
