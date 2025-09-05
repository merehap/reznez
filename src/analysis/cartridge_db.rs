use std::collections::BTreeSet;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use log::info;
use rusqlite::{params, Connection, MappedRows};
use walkdir::WalkDir;

use crate::cartridge::header_db::HeaderDb;
use crate::cartridge::resolved_metadata::ResolvedMetadata;
use crate::config::{Config, GuiType, Opt};
use crate::nes::Nes;

pub fn analyze(rom_base_path: &Path) -> Vec<(PathBuf, ResolvedMetadata)> {
    let rom_paths: BTreeSet<_> = WalkDir::new(rom_base_path)
        .into_iter()
        .map(|entry| entry.unwrap().path().to_path_buf())
        .filter(|path| path.extension() == Some(OsStr::new("nes"))
            && !path.file_stem().unwrap().to_string_lossy().ends_with("#ignored"))
        .collect();

    let header_db = HeaderDb::load();

    let mut all_metadata = Vec::new();
    for rom_path in rom_paths {
        let opt = Opt {
            gui: GuiType::NoGui,
            disable_audio: true,
            prevent_saving: true,
            ..Opt::new(None)
        };

        let config = Config::new(&opt);

        let cartridge = Nes::load_cartridge(&rom_path);
        let nes = Nes::new(&header_db, &config, cartridge);
        log::logger().flush();
        all_metadata.push((rom_path, nes.resolved_metadata().clone()));
    }

    let connection = Connection::open_in_memory().unwrap();
    connection
        .execute(
            "CREATE TABLE cartridges (
            name TEXT NOT NULL,
            mapper INTEGER NOT NULL,
            submapper INTEGER,
            name_table_mirroring TEXT NOT NULL,
            full_hash TEXT NOT NULL,
            prg_rom_hash TEXT NOT NULL,
            chr_rom_hash TEXT NOT NULL,
            prg_rom_size INTEGER NOT NULL,
            prg_work_ram_size INTEGER NOT NULL,
            prg_save_ram_size INTEGER NOT NULL,
            chr_rom_size INTEGER NOT NULL,
            chr_work_ram_size INTEGER NOT NULL,
            chr_save_ram_size INTEGER NOT NULL,
            console_type TEXT NOT NULL,
            region_timing_mode TEXT NOT NULL,
            miscellaneous_rom_count INTEGER NOT NULL,
            default_expansion_device TEXT NOT NULL,
            vs_hardware_type TEXT,
            vs_ppu_type TEXT
        )",
            [],
        )
        .unwrap();
    for (path, metadata) in &all_metadata {
        connection
            .execute(
                "INSERT INTO cartridges VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19)",
                params![
                    path.file_stem().unwrap().to_str().unwrap(),
                    metadata.mapper_number,
                    metadata.submapper_number,
                    metadata.name_table_mirroring.unwrap().to_string(),
                    metadata.full_hash.to_string(),
                    metadata.prg_rom_hash.to_string(),
                    metadata.chr_rom_hash.to_string(),
                    metadata.prg_rom_size.to_string(),
                    metadata.prg_work_ram_size.to_string(),
                    metadata.prg_save_ram_size.to_string(),
                    metadata.chr_rom_size.to_string(),
                    metadata.chr_work_ram_size.to_string(),
                    metadata.chr_save_ram_size.to_string(),
                    metadata.console_type.to_string(),
                    format!("{:?}", metadata.region_timing_mode),
                    metadata.miscellaneous_rom_count.to_string(),
                    format!("{:?}", metadata.default_expansion_device),
                    metadata.vs.clone().map(|vs| format!("{:?}", vs.hardware_type)),
                    metadata.vs.as_ref().map(|vs| format!("{:?}", vs.ppu_type)),
                ],
            )
            .unwrap();
    }

    let db = CartridgeDB { connection };
    let mut select = db
        .connection
        .prepare("SELECT * FROM cartridges ORDER BY mapper ASC")
        .unwrap();

    let cartridge_iter: MappedRows<_> = select
        .query_map([], |row| {
            let r0: String = row.get("name").unwrap();
            let r1: i32 = row.get("mapper").unwrap();
            let r2: String = row.get("name_table_mirroring").unwrap();
            Ok((r0, r1, r2))
        })
        .unwrap();

    cartridge_iter.for_each(|entry| {
        let entry = entry.as_ref().unwrap();
        info!("{} {} {}", entry.0.clone(), entry.1, entry.2.clone());
    });

    all_metadata
}

struct CartridgeDB {
    connection: Connection,
}
