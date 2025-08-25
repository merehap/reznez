use std::path::{Path, PathBuf};

use log::error;

use crate::cartridge::cartridge_metadata::CartridgeMetadata;
use crate::memory::raw_memory::{RawMemory, RawMemoryArray};
use crate::util::unit::KIBIBYTE;

// TODO: Move path and allow_saving elsewhere.
// TODO: Rename? To CartridgeRom? Name depends on if the trainer can be called ROM or not.
// See https://wiki.nesdev.org/w/index.php?title=INES
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct Cartridge {
    path: CartridgePath,
    title: String,
    allow_saving: bool,

    prg_rom: RawMemory,
    chr_rom: RawMemory,
    trainer: Option<RawMemoryArray<512>>,
}

impl Cartridge {
    #[rustfmt::skip]
    pub fn load(path: &Path, header: &CartridgeMetadata, raw_header_and_data: &RawMemory, allow_saving: bool) -> Result<Cartridge, String> {
        let path = CartridgePath(path.to_path_buf());

        let prg_rom_start = 0x10;
        let prg_rom_end = prg_rom_start + header.prg_rom_size().unwrap();
        let Some(prg_rom) = raw_header_and_data.maybe_slice(prg_rom_start..prg_rom_end) else {
            return Err(format!("ROM {} was too short (claimed to have {}KiB PRG ROM).", path.rom_file_name(), header.prg_rom_size().unwrap() / KIBIBYTE));
        };
        let prg_rom = prg_rom.to_raw_memory();

        let chr_rom_start = prg_rom_end;
        let mut chr_rom_end = chr_rom_start + header.chr_rom_size().unwrap();
        let chr_rom = if let Some(rom) = raw_header_and_data.maybe_slice(chr_rom_start..chr_rom_end) {
            rom.to_raw_memory()
        } else {
            error!("ROM {} claimed to have {}KiB CHR ROM, but the ROM was too short.", path.rom_file_name(), header.chr_rom_size().unwrap());
            chr_rom_end = raw_header_and_data.size();
            raw_header_and_data.slice(chr_rom_start..raw_header_and_data.size()).to_raw_memory()
        };

        let title_start = chr_rom_end;
        let title = raw_header_and_data.slice(title_start..raw_header_and_data.size()).to_raw().to_vec();
        let title_length_is_proper = title.is_empty() || title.len() == 127 || title.len() == 128;
        if !title_length_is_proper {
            return Err(format!("Title must be empty or 127 or 128 bytes, but was {} bytes.", title.len()));
        }

        let title = std::str::from_utf8(&title)
            .map_err(|err| err.to_string())?
            .chars()
            .take_while(|&c| c != '\u{0}')
            .collect();

        Ok(Cartridge { path, title, trainer: None, prg_rom, chr_rom, allow_saving })
    }

    pub fn name(&self) -> String {
        self.path.rom_name()
    }

    pub fn path(&self) -> &CartridgePath {
        &self.path
    }

    pub fn prg_rom(&self) -> &RawMemory {
        &self.prg_rom
    }

    pub fn chr_rom(&self) -> &RawMemory {
        &self.chr_rom
    }

    pub fn prg_rom_size(&self) -> u32 {
        self.prg_rom.size()
    }

    pub fn chr_rom_size(&self) -> u32 {
        self.chr_rom.size()
    }

    pub fn allow_saving(&self) -> bool {
        self.allow_saving
    }
}

#[derive(Clone, Debug)]
pub struct CartridgePath(PathBuf);

impl CartridgePath {
    pub fn rom_name(&self) -> String {
        self.0.file_stem().unwrap().to_str().unwrap().to_string()
    }

    pub fn rom_file_name(&self) -> String {
        self.0.file_name().unwrap().to_str().unwrap().to_string()
    }

    pub fn to_prg_save_ram_file_path(&self) -> PathBuf {
        let mut save_path = PathBuf::new();
        save_path.push("saveram");
        save_path.push(self.0.file_stem().unwrap());
        save_path.set_extension("prg.saveram");
        save_path
    }
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct PlayChoice {
    inst_rom: [u8; 8192],
    prom_data: [u8; 16],
    prom_counter_out: [u8; 16],
}