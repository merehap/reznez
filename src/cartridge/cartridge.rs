use std::fmt;
use std::path::{Path, PathBuf};

use log::{info, warn, error};
use splitbits::{splitbits, splitbits_named};

use crate::cartridge::header_db::HeaderDb;
use crate::memory::ppu::chr_memory::AccessOverride;
use crate::memory::raw_memory::{RawMemory, RawMemoryArray};
use crate::ppu::name_table::name_table_mirroring::NameTableMirroring;
use crate::util::unit::KIBIBYTE;

const INES_HEADER_CONSTANT: &[u8] = &[b'N', b'E', b'S', 0x1A];
const PRG_ROM_CHUNK_LENGTH: usize = 16 * KIBIBYTE as usize;
const CHR_ROM_CHUNK_LENGTH: usize = 8 * KIBIBYTE as usize;

#[derive(Clone, Debug)]
pub struct Nes2Fields {
    submapper_number: u8,

    prg_work: u32,
    prg_save: u32,
    chr_work: u32,
    chr_save: u32,
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct RomHeader {
    mapper_number: u16,
    name_table_mirroring: Option<NameTableMirroring>,
    has_persistent_memory: bool,
    console_type: ConsoleType,

    prg_rom_size: u32,
    nes2_fields: Option<Nes2Fields>,
    chr_rom_size: u32,
}

impl RomHeader {
    pub fn parse(header: [u8; 16]) -> Result<Self, String> {
        if &header[0..4] != INES_HEADER_CONSTANT {
            return Err(format!(
                "Cannot load non-iNES ROM. Found {:?} but need {:?}.",
                &header[0..4],
                INES_HEADER_CONSTANT,
            ));
        }

        let prg_rom_chunk_count = header[4] as u32;
        let chr_rom_chunk_count = header[5] as u32;

        let (lower_mapper_number, four_screen, trainer_enabled, has_persistent_memory, vertical_mirroring) =
            splitbits_named!(header[6], "llllftpv");
        let (upper_mapper_number, ines2, play_choice_enabled, vs_unisystem_enabled) =
            splitbits_named!(header[7], "uuuuiipv");
        let ines2_present = ines2 == 0b10;

        if trainer_enabled {
            return Err("Trainer isn't implemented yet.".to_string());
        }

        let mut mapper_number = u16::from((upper_mapper_number << 4) | lower_mapper_number);
        let mut ram_sizes = None;
        if ines2_present {
            mapper_number |= u16::from(header[8] & 0b1111) << 8;
            let submapper_number = header[8] >> 4;
            let prg_sizes = splitbits!(min=u32, header[10], "sssswwww");
            let prg_work = if prg_sizes.w > 0 { 64 << prg_sizes.w } else { 0 };
            let prg_save = if prg_sizes.s > 0 { 64 << prg_sizes.s } else { 0 };

            // FIXME: This should be from rom[11], not rom[10].
            let chr_sizes = splitbits!(min=u32, header[10], "sssswwww");
            let chr_work = if chr_sizes.w > 0 { 64 << chr_sizes.w } else { 0 };
            let chr_save = if chr_sizes.s > 0 { 64 << chr_sizes.s } else { 0 };

            ram_sizes = Some(Nes2Fields { submapper_number, prg_work, prg_save, chr_work, chr_save })
        }

        if play_choice_enabled {
            return Err("PlayChoice isn't implemented yet.".to_string());
        }

        if vs_unisystem_enabled {
            return Err("VS Unisystem isn't implemented yet.".to_string());
        }

        let name_table_mirroring = if four_screen {
            // Four screen mirroring isn't a real mirroring, the mapper will have to define what it means.
            None
        } else if vertical_mirroring {
            Some(NameTableMirroring::VERTICAL)
        } else {
            Some(NameTableMirroring::HORIZONTAL)
        };

        Ok(RomHeader {
            mapper_number,
            name_table_mirroring,
            has_persistent_memory,
            console_type: ConsoleType::Nes,
            prg_rom_size: prg_rom_chunk_count * PRG_ROM_CHUNK_LENGTH as u32,
            chr_rom_size: chr_rom_chunk_count * CHR_ROM_CHUNK_LENGTH as u32,
            nes2_fields: ram_sizes,
        })
    }

    fn chr_present(&self) -> bool {
        if self.chr_rom_size > 0 {
            return true;
        }

        if let Some(ram_sizes) = &self.nes2_fields {
            ram_sizes.chr_work > 0 || ram_sizes.chr_save > 0
        } else {
            false
        }
    }
}

// See https://wiki.nesdev.org/w/index.php?title=INES
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct Cartridge {
    path: CartridgePath,
    header: RomHeader,
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

        let raw_header = rom.slice(0x0..0x10).to_raw().try_into()
            .map_err(|err| format!("ROM file to have a 16 byte header. {err}"))?;
        let header = RomHeader::parse(raw_header)?;

        let prg_rom_start = 0x10;
        let prg_rom_end = prg_rom_start + header.prg_rom_size;
        let prg_rom = rom.maybe_slice(prg_rom_start..prg_rom_end)
            .unwrap_or_else(|| {
                panic!("ROM {} was too short (claimed to have {}KiB PRG ROM).", path.rom_file_name(), header.prg_rom_size / KIBIBYTE);
            })
            .to_raw_memory();

        let chr_rom_start = prg_rom_end;
        let mut chr_rom_end = chr_rom_start + header.chr_rom_size;
        let chr_rom = if let Some(rom) = rom.maybe_slice(chr_rom_start..chr_rom_end) {
            rom.to_raw_memory()
        } else {
            error!("ROM {} claimed to have {}KiB CHR ROM, but the ROM was too short.", path.rom_file_name(), header.chr_rom_size);
            chr_rom_end = rom.size();
            rom.slice(chr_rom_start..rom.size()).to_raw_memory()
        };

        let mut submapper_number = None;
        let mut prg_work_ram_size = None;
        let mut prg_save_ram_size = None;
        let mut chr_work_ram_size = None;
        let mut chr_save_ram_size = None;
        if let Some(nes2_fields) = &header.nes2_fields {
            submapper_number = Some(nes2_fields.submapper_number);
            prg_work_ram_size = Some(nes2_fields.prg_work);
            prg_save_ram_size = Some(nes2_fields.prg_save);
            chr_work_ram_size = Some(nes2_fields.chr_work);
            chr_save_ram_size = Some(nes2_fields.chr_save);
        }

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

        let full_hash = crc32fast::hash(rom.as_slice());
        let prg_hash = crc32fast::hash(cartridge.prg_rom.as_slice());
        if let Some(header) = header_db.header_from_db(&cartridge, full_hash, prg_hash, cartridge.header.mapper_number, cartridge.submapper_number) {
            if cartridge.header.mapper_number != header.mapper_number {
                warn!("Mapper number in ROM ({}) does not match the one in the DB ({}).",
                    cartridge.header.mapper_number, header.mapper_number);
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
                    header_db.missing_submapper_number(full_hash, prg_hash) && cartridge.header.mapper_number == number {

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
        self.header.mapper_number
    }

    pub fn submapper_number(&self) -> Option<u8> {
        self.submapper_number
    }

    pub fn name_table_mirroring(&self) -> Option<NameTableMirroring> {
        self.header.name_table_mirroring
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
        write!(f, "Mapper: {}", self.header.mapper_number)?;
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
        writeln!(f, "Console: {}", self.header.console_type)?;

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
pub enum ConsoleType {
    Nes,
    VsUnisystem,
    PlayChoice10(Box<PlayChoice>),
    Extended,
}

impl fmt::Display for ConsoleType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self.clone() {
            ConsoleType::Nes => "NES",
            ConsoleType::VsUnisystem => "VS Unisystem",
            ConsoleType::PlayChoice10(_) => "Play Choice 10",
            ConsoleType::Extended => "Extended",
        };

        write!(f, "{text}")
    }
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct PlayChoice {
    inst_rom: [u8; 8192],
    prom_data: [u8; 16],
    prom_counter_out: [u8; 16],
}

#[cfg(test)]
pub mod test_data {
    use super::*;

    #[allow(dead_code)]
    pub fn cartridge() -> Cartridge {
        let mut prg_rom = vec![0xEA; PRG_ROM_CHUNK_LENGTH];
        // Overwrite the NMI/RESET/IRQ Vectors so they doesn't point to ROM.
        // This allows injection of custom instructions for testing.
        prg_rom[PRG_ROM_CHUNK_LENGTH - 6] = 0x00;
        prg_rom[PRG_ROM_CHUNK_LENGTH - 5] = 0x02;
        prg_rom[PRG_ROM_CHUNK_LENGTH - 4] = 0x00;
        prg_rom[PRG_ROM_CHUNK_LENGTH - 3] = 0x02;
        prg_rom[PRG_ROM_CHUNK_LENGTH - 2] = 0x00;
        prg_rom[PRG_ROM_CHUNK_LENGTH - 1] = 0x02;

        let mut path = PathBuf::new();
        path.set_file_name("Test");
        let path = CartridgePath(path);

        Cartridge {
            path,
            header: RomHeader {
                mapper_number: 0,
                name_table_mirroring: Some(NameTableMirroring::HORIZONTAL),
                has_persistent_memory: false,
                console_type: ConsoleType::Nes,
                prg_rom_size: prg_rom.len() as u32,
                chr_rom_size: CHR_ROM_CHUNK_LENGTH as u32,
                nes2_fields: None,
            },
            submapper_number: None,

            trainer: None,

            prg_rom: RawMemory::from_vec(prg_rom),
            prg_work_ram: RawMemory::new(0),
            prg_save_ram: RawMemory::new(0),
            chr_rom: RawMemory::new(CHR_ROM_CHUNK_LENGTH as u32),
            chr_work_ram: RawMemory::new(0),
            chr_save_ram: RawMemory::new(0),
            allow_saving: true,

            title: "Test ROM".to_string(),
        }
    }
}
