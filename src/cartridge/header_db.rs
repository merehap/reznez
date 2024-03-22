use std::collections::BTreeMap;

pub struct HeaderDb {
    full_data_by_crc32: BTreeMap<u32, Header>,
    prg_rom_by_crc32: BTreeMap<u32, Header>,
}

impl HeaderDb {
    pub fn load() -> HeaderDb {
        let text = include_str!("../../nes20db.xml");
        let doc = roxmltree::Document::parse(text).unwrap();
        let games = doc.root().descendants().filter(|n| n.tag_name().name() == "game");

        let mut header_db = HeaderDb {
            full_data_by_crc32: BTreeMap::new(),
            prg_rom_by_crc32: BTreeMap::new(),
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
            header_db.full_data_by_crc32.insert(u32::from_str_radix(rom_crc32, 16).unwrap(), header);
            let prg_rom_crc32 = read_attribute(game, "prgrom", "crc32").unwrap();
            header_db.prg_rom_by_crc32.insert(u32::from_str_radix(prg_rom_crc32, 16).unwrap(), header);
        }

        header_db
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
struct Header {
    prg_rom_size: u32,
    prg_ram_size: u32,
    chr_rom_size: u32,
    chr_ram_size: u32,
    mapper_number: u16,
    submapper_number: u8,
}
