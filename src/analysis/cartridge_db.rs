use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsStr;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use itertools::Itertools;
use log::{error, info};
use rusqlite::{params, Connection, Result, MappedRows};
use walkdir::WalkDir;

use crate::cartridge::Cartridge;

pub fn analyze(rom_base_path: &Path) {
    let rom_paths: BTreeSet<_> = WalkDir::new(rom_base_path)
        .into_iter()
        .map(|entry| entry.unwrap().path().to_path_buf())
        .filter(|path| path.extension() == Some(OsStr::new("nes")))
        .collect();

    let mut cartridges = Vec::new();
    for rom_path in rom_paths {
        let mut rom = Vec::new();
        File::open(rom_path.clone())
            .unwrap()
            .read_to_end(&mut rom)
            .unwrap();
        let file_name = rom_path.file_stem().unwrap().to_str().unwrap().to_string();
        match Cartridge::load(file_name, &rom) {
            Err(err) => error!("Failed to load rom {}. {}", rom_path.display(), err),
            Ok(cartridge) => cartridges.push(cartridge),
        }
    }

    let connection = Connection::open_in_memory().unwrap();
    connection.execute(
        "CREATE TABLE cartridges (
            name TEXT NOT NULL,
            mapper INTEGER NOT NULL,
            mirroring TEXT NOT NULL
        )",
        [],
    ).unwrap();
    for cartridge in cartridges {
        connection.execute(
            "INSERT INTO cartridges VALUES (?1, ?2, ?3)",
            params![
                cartridge.name(),
                cartridge.mapper_number(),
                format!("{:?}", cartridge.name_table_mirroring()),
            ],
        ).unwrap();
    }

    let db = CartridgeDB {connection};
    let mut select = db.connection.prepare(
        "SELECT name, mapper, mirroring FROM cartridges ORDER BY mapper ASC"
    ).unwrap();

    let cartridge_iter: MappedRows<_> = select.query_map([], |row| {
        Ok((
                row.get(0).unwrap(),
                row.get(1).unwrap(),
                row.get(2).unwrap(),
        )) : Result<(String, i32, String)>
    }).unwrap();

    let cartridge_iter = cartridge_iter.map(|entry| {
        let entry = entry.as_ref().unwrap();
        (entry.0.clone(), entry.1.clone(), entry.2.clone())
    });

    let grouped_cartridges: BTreeMap<i32, Vec<(String, i32, String)>> = cartridge_iter
        .into_group_map_by(|(_, mapper_number, _)| *mapper_number)
        .into_iter()
        .collect();
    for (mapper_number, group) in grouped_cartridges.iter() {
        info!("{}", mapper_number);
        for (name, _, mirroring) in group {
            info!("\t{}: {} mirroring", name, mirroring);
        }
    }
}

struct CartridgeDB {
    connection: Connection,
}