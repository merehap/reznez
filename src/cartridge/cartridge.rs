use std::fmt;
use std::path::{Path, PathBuf};

use log::{info, warn, error};

use crate::cartridge::cartridge_header::CartridgeHeader;
use crate::cartridge::header_db::HeaderDb;
use crate::memory::ppu::chr_memory::AccessOverride;
use crate::memory::raw_memory::{RawMemory, RawMemoryArray};
use crate::ppu::name_table::name_table_mirroring::NameTableMirroring;
use crate::util::unit::KIBIBYTE;

// See https://wiki.nesdev.org/w/index.php?title=INES
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct Cartridge {
    path: CartridgePath,
    header: CartridgeHeader,
    submapper_number: Option<u8>,
    title: String,

    trainer: Option<RawMemoryArray<512>>,
    prg_rom: RawMemory,
    prg_work_ram: RawMemory,
    prg_save_ram: RawMemory,
    chr_rom: RawMemory,
    chr_work_ram: RawMemory,
    chr_save_ram: RawMemory,
    allow_saving: bool,
}

impl Cartridge {
    #[rustfmt::skip]
    pub fn load(path: &Path, rom: &RawMemory, header_db: &HeaderDb, allow_saving: bool) -> Result<Cartridge, String> {
        let path = CartridgePath(path.to_path_buf());

        let full_hash = crc32fast::hash(rom.as_slice());
        let raw_header = rom.slice(0x0..0x10).to_raw().try_into()
            .map_err(|err| format!("ROM file to have a 16 byte header. {err}"))?;
        let mut header = CartridgeHeader::parse(raw_header, full_hash)?;

        let prg_rom_start = 0x10;
        let prg_rom_end = prg_rom_start + header.prg_rom_size().unwrap();
        let prg_rom = rom.maybe_slice(prg_rom_start..prg_rom_end)
            .unwrap_or_else(|| {
                panic!("ROM {} was too short (claimed to have {}KiB PRG ROM).", path.rom_file_name(), header.prg_rom_size().unwrap() / KIBIBYTE);
            })
            .to_raw_memory();
        let prg_rom_hash = crc32fast::hash(prg_rom.as_slice());
        header.set_prg_rom_hash(prg_rom_hash);

        let chr_rom_start = prg_rom_end;
        let mut chr_rom_end = chr_rom_start + header.chr_rom_size().unwrap();
        let chr_rom = if let Some(rom) = rom.maybe_slice(chr_rom_start..chr_rom_end) {
            rom.to_raw_memory()
        } else {
            error!("ROM {} claimed to have {}KiB CHR ROM, but the ROM was too short.", path.rom_file_name(), header.chr_rom_size().unwrap());
            chr_rom_end = rom.size();
            rom.slice(chr_rom_start..rom.size()).to_raw_memory()
        };

        let submapper_number = header.submapper_number();
        let prg_work_ram_size = header.prg_work_ram_size();
        let prg_save_ram_size = header.prg_save_ram_size();
        let chr_work_ram_size = header.chr_work_ram_size();
        let chr_save_ram_size = header.chr_save_ram_size();

        let title_start = chr_rom_end;
        let title = rom.slice(title_start..rom.size()).to_raw().to_vec();
        let title_length_is_proper = title.is_empty() || title.len() == 127 || title.len() == 128;
        if !title_length_is_proper {
            return Err(format!("Title must be empty or 127 or 128 bytes, but was {} bytes.", title.len()));
        }

        let title = std::str::from_utf8(&title)
            .map_err(|err| err.to_string())?
            .chars()
            .take_while(|&c| c != '\u{0}')
            .collect();

        let mut cartridge = Cartridge {
            path,
            header,
            submapper_number,
            title,

            trainer: None,
            prg_rom,
            prg_work_ram: RawMemory::new(prg_work_ram_size.unwrap_or(0)),
            prg_save_ram: RawMemory::new(prg_save_ram_size.unwrap_or(0)),
            chr_rom,
            chr_work_ram: RawMemory::new(chr_work_ram_size.unwrap_or(0)),
            chr_save_ram: RawMemory::new(chr_save_ram_size.unwrap_or(0)),
            allow_saving,
        };

        let cartridge_mapper_number = cartridge.header.mapper_number().unwrap();
        if let Some(header) = header_db.header_from_db(&cartridge, full_hash, prg_rom_hash, cartridge_mapper_number, cartridge.submapper_number) {
            if cartridge_mapper_number != header.mapper_number {
                warn!("Mapper number in ROM ({}) does not match the one in the DB ({}).",
                    cartridge_mapper_number, header.mapper_number);
            }

            assert_eq!(cartridge.prg_rom.size(), header.prg_rom_size);
            if cartridge.chr_rom.size() != header.chr_rom_size {
                warn!("CHR ROM size in cartridge did not match size in header DB.");
            }

            cartridge.submapper_number = Some(header.submapper_number);
            cartridge.prg_work_ram = RawMemory::new(header.prg_ram_size);
            cartridge.prg_save_ram = RawMemory::new(header.prg_nvram_size);
            cartridge.chr_work_ram = RawMemory::new(header.chr_ram_size);
            cartridge.chr_save_ram = RawMemory::new(header.chr_nvram_size);
        } else {
            warn!("ROM not found in header database.");
            if !cartridge.header.chr_present() {
                // If no CHR data is provided, add 8KiB of CHR RAM.
                cartridge.chr_work_ram = RawMemory::new(8 * KIBIBYTE);
            }

            if let Some((number, sub_number, data_hash, prg_hash)) =
                    header_db.missing_submapper_number(full_hash, prg_rom_hash) && cartridge_mapper_number == number {

                info!("Using override submapper for this ROM. Full hash: {data_hash} , PRG hash: {prg_hash}");
                cartridge.submapper_number = Some(sub_number);
            }
        }

        Ok(cartridge)
    }

    pub fn name(&self) -> String {
        self.path.rom_name()
    }

    pub fn path(&self) -> &CartridgePath {
        &self.path
    }

    pub fn mapper_number(&self) -> u16 {
        self.header.mapper_number().unwrap()
    }

    pub fn submapper_number(&self) -> Option<u8> {
        self.submapper_number
    }

    pub fn name_table_mirroring(&self) -> Option<NameTableMirroring> {
        self.header.name_table_mirroring()
    }

    pub fn prg_rom(&self) -> &RawMemory {
        &self.prg_rom
    }

    pub fn prg_work_ram(&self) -> &RawMemory {
        &self.prg_work_ram
    }

    pub fn chr_rom(&self) -> &RawMemory {
        &self.chr_rom
    }

    pub fn chr_ram(&self) -> RawMemory {
        // FIXME
        RawMemory::new(self.chr_work_ram.size() + self.chr_save_ram.size())
    }

    pub fn set_prg_rom_at(&mut self, index: u32, value: u8) {
        self.prg_rom[index] = value;
    }

    pub fn prg_rom_size(&self) -> u32 {
        self.prg_rom.size()
    }

    pub fn prg_work_ram_size(&self) -> u32 {
        self.prg_work_ram.size()
    }

    pub fn prg_save_ram_size(&self) -> u32 {
        self.prg_save_ram.size()
    }

    pub fn prg_rom_forced(&self) -> bool {
        self.prg_work_ram.is_empty() && self.prg_save_ram.is_empty()
    }

    pub fn chr_rom_size(&self) -> u32 {
        self.chr_rom.size()
    }

    pub fn chr_work_ram_size(&self) -> u32 {
        self.chr_work_ram.size()
    }

    pub fn chr_save_ram_size(&self) -> u32 {
        self.chr_save_ram.size()
    }

    pub fn chr_access_override(&self) -> Option<AccessOverride> {
        if self.chr_rom.is_empty() {
            Some(AccessOverride::ForceRam)
        } else if self.chr_work_ram.is_empty() && self.chr_save_ram.is_empty() {
            Some(AccessOverride::ForceRom)
        } else {
            None
        }
    }

    pub fn allow_saving(&self) -> bool {
        self.allow_saving
    }
}

impl fmt::Display for Cartridge {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Mapper: {}", self.header.mapper_number().unwrap())?;
        if let Some(submapper_number) = self.submapper_number {
            write!(f, " (Submapper: {submapper_number})")?;
        }

        writeln!(f)?;
        writeln!(f, "PRG ROM: {:4}KiB, WorkRAM: {:4}KiB, SaveRAM: {:4}KiB",
            self.prg_rom.size() / KIBIBYTE,
            self.prg_work_ram.size() / KIBIBYTE,
            self.prg_save_ram.size() / KIBIBYTE,
        )?;
        writeln!(f, "CHR ROM: {:4}KiB, WorkRAM: {:4}KiB, SaveRAM: {:4}KiB",
            self.chr_rom.size() / KIBIBYTE,
            self.chr_work_ram.size() / KIBIBYTE,
            self.chr_save_ram.size() / KIBIBYTE,
        )?;
        writeln!(f, "Console: {}", self.header.console_type().unwrap())?;

        Ok(())
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