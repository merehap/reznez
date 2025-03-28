use std::fmt;

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

// See https://wiki.nesdev.org/w/index.php?title=INES
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct Cartridge {
    name: String,

    mapper_number: u16,
    submapper_number: u8,
    name_table_mirroring: Option<NameTableMirroring>,
    has_persistent_memory: bool,
    ines2_present: bool,

    trainer: Option<RawMemoryArray<512>>,

    prg_access_override: Option<AccessOverride>,
    chr_access_override: Option<AccessOverride>,

    prg_rom: RawMemory,
    prg_ram_size: u32,
    prg_nvram_size: u32,
    chr_rom: RawMemory,
    chr_ram_size: u32,
    chr_nvram_size: u32,

    console_type: ConsoleType,
    title: String,
}

impl Cartridge {
    #[rustfmt::skip]
    pub fn load(name: String, rom: &RawMemory, header_db: &HeaderDb) -> Result<Cartridge, String> {
        if rom.slice(0..4).to_raw() != INES_HEADER_CONSTANT {
            return Err(format!(
                "Cannot load non-iNES ROM. Found {:?} but need {:?}.",
                rom.slice(0..4),
                INES_HEADER_CONSTANT,
            ));
        }

        let prg_rom_chunk_count = rom[4] as u32;
        let chr_rom_chunk_count = rom[5] as u32;

        let (lower_mapper_number, four_screen, trainer_enabled, has_persistent_memory, vertical_mirroring) =
            splitbits_named!(rom[6], "llllftpv");
        let (upper_mapper_number, ines2, play_choice_enabled, vs_unisystem_enabled) =
            splitbits_named!(rom[7], "uuuuiipv");
        let ines2_present = ines2 == 0b10;

        if trainer_enabled {
            return Err("Trainer isn't implemented yet.".to_string());
        }

        let mut mapper_number = u16::from((upper_mapper_number << 4) | lower_mapper_number);
        let mut submapper_number = 0;
        let mut prg_ram_size = 0;
        let mut prg_nvram_size = 0;
        let mut chr_ram_size = 0;
        let mut chr_nvram_size = 0;
        if ines2_present {
            mapper_number |= u16::from(rom[8] & 0b1111) << 8;
            submapper_number = rom[8] >> 4;
            let prg_sizes = splitbits!(min=u32, rom[10], "eeeepppp");
            match (prg_sizes.e, prg_sizes.p) {
                (0, 0) => { /* Do nothing. */ }
                (0, 1..) => prg_ram_size = 64 << prg_sizes.p,
                (1.., 0) => prg_nvram_size = 64 << prg_sizes.e,
                (1.., 1..) => panic!("Both EEPROM and PRGRAM are present. Not sure what to do."),
            }

            let chr_sizes = splitbits!(min=u32, rom[10], "nnnnpppp");
            match (chr_sizes.n, chr_sizes.p) {
                (0, 0) => { /* Do nothing. */ }
                (0, 1..) => chr_ram_size = 64 << chr_sizes.p,
                (1.., 0) => chr_nvram_size = 64 << chr_sizes.n,
                (1.., 1..) => panic!("Both CHR NVRAM and CHRRAM are present. Not sure what to do."),
            }
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

        let prg_rom_start = 0x10;
        let prg_rom_end = prg_rom_start + PRG_ROM_CHUNK_LENGTH as u32 * prg_rom_chunk_count;
        let prg_rom = rom.maybe_slice(prg_rom_start..prg_rom_end)
            .unwrap_or_else(
                || panic!("ROM {name} was too short (claimed to have {prg_rom_chunk_count} PRG chunks)."));

        let mut prg_access_override = None;
        if prg_ram_size == 0 && prg_nvram_size == 0 {
            prg_access_override = Some(AccessOverride::ForceRom);
        }

        let chr_rom_start = prg_rom_end;
        let mut chr_rom_end = chr_rom_start + CHR_ROM_CHUNK_LENGTH as u32 * chr_rom_chunk_count;
        let mut chr_access_override = None;
        let chr_rom = if let Some(rom) = rom.maybe_slice(chr_rom_start..chr_rom_end) {
            rom.to_raw_memory()
        } else {
            error!("ROM {} claimed to have {} CHR ROM chunks, but the ROM was too short.",
                name, chr_rom_chunk_count);
            chr_rom_end = rom.size();
            rom.slice(chr_rom_start..rom.size()).to_raw_memory()
        };

        if chr_rom.is_empty() {
            if chr_ram_size == 0 && chr_nvram_size == 0 {
                // If no CHR data is provided, add 8KiB of CHR RAM and allow writing to read-only layouts.
                chr_access_override = Some(AccessOverride::ForceRam);
                chr_ram_size = 8 * KIBIBYTE;
            }
        } else if chr_ram_size > 0 || chr_nvram_size > 0 {
            // It's not yet clear what to do in this case, but this passes M1_P128K_C128K_W8K.
            warn!("Both CHR ROM and CHR RAM were specified. Disabling writability.");
            chr_access_override = Some(AccessOverride::ForceRom)
        } else {
            // ROM provided but no RAM.
            chr_access_override = Some(AccessOverride::ForceRom)
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
            name,

            mapper_number,
            submapper_number,
            name_table_mirroring,
            has_persistent_memory,
            ines2_present,

            trainer: None,

            prg_access_override,
            chr_access_override,
            prg_rom: prg_rom.to_raw_memory(),
            prg_ram_size,
            prg_nvram_size,
            chr_rom,
            chr_ram_size,
            chr_nvram_size,
            console_type: ConsoleType::Nes,
            title,
        };

        if let Some(header) = header_db.header_from_db(&cartridge, rom, &prg_rom, mapper_number, submapper_number) {
            if cartridge.mapper_number != header.mapper_number {
                warn!("Mapper number in ROM ({}) does not match the one in the DB ({}).",
                    cartridge.mapper_number, header.mapper_number);
            }

            assert_eq!(prg_rom.size(), header.prg_rom_size);
            if cartridge.chr_rom.size() != header.chr_rom_size {
                warn!("CHR ROM size in cartridge did not match size in header DB.");
            }

            cartridge.submapper_number = header.submapper_number;
            cartridge.prg_ram_size = header.prg_ram_size;
            cartridge.prg_nvram_size = header.prg_nvram_size;
            cartridge.chr_ram_size = chr_ram_size;
            cartridge.chr_nvram_size = header.chr_nvram_size;
        } else {
            warn!("ROM not found in header database.");
            if let Some((number, sub_number, data_hash, prg_hash)) =
                    header_db.missing_submapper_number(rom, &prg_rom) && cartridge.mapper_number == number {

                info!("Using override submapper for this ROM. Full hash: {data_hash} , PRG hash: {prg_hash}");
                cartridge.submapper_number = sub_number;
            }
        }

        Ok(cartridge)
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn mapper_number(&self) -> u16 {
        self.mapper_number
    }

    pub fn submapper_number(&self) -> u8 {
        self.submapper_number
    }

    pub fn name_table_mirroring(&self) -> Option<NameTableMirroring> {
        self.name_table_mirroring
    }

    pub fn prg_rom(&self) -> &RawMemory {
        &self.prg_rom
    }

    pub fn chr_rom(&self) -> &RawMemory {
        &self.chr_rom
    }

    pub fn chr_ram(&self) -> RawMemory {
        RawMemory::new(self.chr_ram_size + self.chr_nvram_size)
    }

    pub fn set_prg_rom_at(&mut self, index: u32, value: u8) {
        self.prg_rom[index] = value;
    }

    pub fn prg_rom_size(&self) -> u32 {
        self.prg_rom.size()
    }

    pub fn prg_ram_size(&self) -> u32 {
        self.prg_ram_size
    }

    pub fn prg_nvram_size(&self) -> u32 {
        self.prg_nvram_size
    }

    pub fn prg_access_override(&self) -> Option<AccessOverride> {
        self.prg_access_override
    }

    pub fn chr_rom_size(&self) -> u32 {
        self.chr_rom.size()
    }

    pub fn chr_ram_size(&self) -> u32 {
        self.chr_ram_size
    }

    pub fn chr_nvram_size(&self) -> u32 {
        self.chr_nvram_size
    }

    pub fn chr_access_override(&self) -> Option<AccessOverride> {
        self.chr_access_override
    }
}

impl fmt::Display for Cartridge {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mirroring = self.name_table_mirroring
            .map(|m| m.to_string())
            .unwrap_or("FourScreen".to_string());
        writeln!(f, "Mapper: {}", self.mapper_number)?;
        writeln!(f, "Submapper: {}", self.submapper_number)?;
        writeln!(f, "Name table mirroring: {mirroring}")?;
        writeln!(f, "Persistent memory: {}", self.has_persistent_memory)?;
        writeln!(f, "iNES2 present: {}", self.ines2_present)?;

        writeln!(f, "Trainer present: {}", self.trainer.is_some())?;
        writeln!(f, "PRG ROM size: {}KiB", self.prg_rom.size() / KIBIBYTE)?;
        writeln!(f, "PRG RAM size: {}KiB", self.prg_ram_size / KIBIBYTE)?;
        writeln!(f, "PRG NVRAM size: {}KiB", self.prg_nvram_size / KIBIBYTE)?;
        writeln!(f, "CHR ROM size: {}KiB", self.chr_rom.size() / KIBIBYTE)?;
        writeln!(f, "CHR RAM size: {}KiB", self.chr_ram_size / KIBIBYTE)?;
        writeln!(f, "CHR NVRAM size: {}KiB", self.chr_nvram_size / KIBIBYTE)?;
        writeln!(f, "Console type: {:?}", self.console_type)?;
        writeln!(f, "Title: {:?}", self.title)?;

        Ok(())
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

        Cartridge {
            name: "Test".to_string(),
            mapper_number: 0,
            submapper_number: 0,
            name_table_mirroring: Some(NameTableMirroring::HORIZONTAL),
            has_persistent_memory: false,
            ines2_present: false,

            trainer: None,

            prg_access_override: None,
            chr_access_override: None,

            prg_rom: RawMemory::from_vec(prg_rom),
            prg_ram_size: 0,
            prg_nvram_size: 0,
            chr_rom: RawMemory::new(CHR_ROM_CHUNK_LENGTH as u32),
            chr_ram_size: 0,
            chr_nvram_size: 0,
            console_type: ConsoleType::Nes,
            title: "Test ROM".to_string(),
        }
    }
}
