use std::fmt;

use log::{info, warn, error};

use crate::cartridge::header_db::{HeaderDb, Header};
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
    name_table_mirroring: NameTableMirroring,
    has_persistent_memory: bool,
    ripper_name: String,
    ines2: Option<INes2>,

    trainer: Option<RawMemoryArray<512>>,

    prg_rom: RawMemory,
    prg_ram_size: u32,
    chr_rom: RawMemory,
    chr_ram_size: u32,

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

        let lower_mapper_number   = rom[6] & 0b1111_0000;
        let four_screen           = rom[6] & 0b0000_1000 != 0;
        let trainer_enabled       = rom[6] & 0b0000_0100 != 0;
        let has_persistent_memory = rom[6] & 0b0000_0010 != 0;
        let vertical_mirroring    = rom[6] & 0b0000_0001 != 0;

        let upper_mapper_number   = rom[7] & 0b1111_0000;
        let ines2_present         = rom[7] & 0b0000_1100 == 0b0000_1100;
        let play_choice_enabled   = rom[7] & 0b0000_0010 != 0;
        let vs_unisystem_enabled  = rom[7] & 0b0000_0001 != 0;

        let ripper_name: String = std::str::from_utf8(rom.slice(8..15).to_raw())
            .map_err(|err| err.to_string())?
            .chars()
            .map(|c| if c.is_ascii_graphic() {c} else {'~'})
            .collect();

        if trainer_enabled {
            return Err("Trainer isn't implemented yet.".to_string());
        }

        if ines2_present {
            return Err("iNES2 isn't implemented yet.".to_string());
        }

        if play_choice_enabled {
            return Err("PlayChoice isn't implemented yet.".to_string());
        }

        if vs_unisystem_enabled {
            return Err("VS Unisystem isn't implemented yet.".to_string());
        }

        let mapper_number = u16::from(upper_mapper_number | (lower_mapper_number >> 4));
        let name_table_mirroring = match (four_screen, vertical_mirroring) {
            (true, _) => NameTableMirroring::FourScreen,
            (_, false) => NameTableMirroring::Horizontal,
            (_, true) => NameTableMirroring::Vertical,
        };

        let prg_rom_start = 0x10;
        let prg_rom_end = prg_rom_start + PRG_ROM_CHUNK_LENGTH as u32 * prg_rom_chunk_count;
        let prg_rom = rom.maybe_slice(prg_rom_start..prg_rom_end)
            .unwrap_or_else(
                || panic!("ROM {name} was too short (claimed to have {prg_rom_chunk_count} PRG chunks)."));

        let chr_rom_start = prg_rom_end;
        let mut chr_rom_end = chr_rom_start + CHR_ROM_CHUNK_LENGTH as u32 * chr_rom_chunk_count;
        let chr_rom;
        if let Some(chr) = rom.maybe_slice(chr_rom_start..chr_rom_end) {
            chr_rom = chr;
        } else {
            error!("ROM {} claimed to have {} CHR chunks, but the ROM was too short.",
                name, chr_rom_chunk_count);
            chr_rom_end = rom.size();
            chr_rom = rom.slice(chr_rom_start..rom.size());
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
            submapper_number: 0,
            name_table_mirroring,
            has_persistent_memory,
            ripper_name,
            ines2: None,

            trainer: None,
            prg_rom: prg_rom.to_raw_memory(),
            prg_ram_size: 0,
            chr_rom: chr_rom.to_raw_memory(),
            chr_ram_size: 0,
            console_type: ConsoleType::Nes,
            title,
        };

        if let Some((mapper_number, submapper_number, data_hash, prg_hash)) = header_db.missing_submapper_number(rom, &prg_rom)
                && mapper_number == cartridge.mapper_number {
            info!("Using override submapper for this ROM. Data hash: {data_hash} , PRG hash: {prg_hash}");
            cartridge.submapper_number = submapper_number;
        } else if let Some(Header { prg_rom_size, prg_ram_size, chr_rom_size, chr_ram_size, mapper_number, submapper_number }) =
                header_db.header_from_data(rom) {
            if cartridge.mapper_number != mapper_number {
                warn!("Mapper number in ROM ({}) does not match the one in the DB {mapper_number}.", cartridge.mapper_number);
            }

            assert_eq!(prg_rom.size(), prg_rom_size);
            assert_eq!(chr_rom.size(), chr_rom_size);
            cartridge.submapper_number = submapper_number;
            cartridge.prg_ram_size = prg_ram_size;
            cartridge.chr_ram_size = chr_ram_size;
        } else if let Some(Header { prg_rom_size, prg_ram_size, chr_rom_size, chr_ram_size, mapper_number, submapper_number }) =
                header_db.header_from_prg_rom(&prg_rom) {
            if cartridge.mapper_number != mapper_number {
                warn!("Mapper number in ROM ({}) does not match the one in the DB {mapper_number}.", cartridge.mapper_number);
            }

            assert_eq!(prg_rom.size(), prg_rom_size);
            assert_eq!(chr_rom.size(), chr_rom_size);
            cartridge.submapper_number = submapper_number;
            cartridge.prg_ram_size = prg_ram_size;
            cartridge.chr_ram_size = chr_ram_size;
        } else {
            warn!("ROM not found in header database.");
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

    pub fn name_table_mirroring(&self) -> NameTableMirroring {
        self.name_table_mirroring
    }

    pub fn prg_rom(&self) -> &RawMemory {
        &self.prg_rom
    }

    pub fn chr_rom(&self) -> &RawMemory {
        &self.chr_rom
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

    pub fn chr_ram_size(&self) -> u32 {
        self.chr_ram_size
    }
}

impl fmt::Display for Cartridge {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Mapper: {}", self.mapper_number)?;
        writeln!(f, "Submapper: {}", self.submapper_number)?;
        writeln!(f, "Nametable mirroring: {:?}", self.name_table_mirroring)?;
        writeln!(f, "Persistent memory: {}", self.has_persistent_memory)?;
        writeln!(f, "Ripper: {}", self.ripper_name)?;
        writeln!(f, "iNES2 present: {}", self.ines2.is_some())?;

        writeln!(f, "Trainer present: {}", self.trainer.is_some())?;
        writeln!(f, "PRG ROM size: {}KiB", self.prg_rom.size() / KIBIBYTE)?;
        writeln!(f, "CHR ROM size: {}KiB", self.chr_rom.size() / KIBIBYTE)?;
        writeln!(f, "Console type: {:?}", self.console_type)?;
        writeln!(f, "Title: {:?}", self.title)?;

        Ok(())
    }
}

#[derive(Clone, Debug)]
struct INes2 {}

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
            name_table_mirroring: NameTableMirroring::Horizontal,
            has_persistent_memory: false,
            ripper_name: "Test Ripper".to_string(),
            ines2: None,

            trainer: None,
            prg_rom: RawMemory::from_vec(prg_rom),
            prg_ram_size: 0,
            chr_rom: RawMemory::new(CHR_ROM_CHUNK_LENGTH as u32),
            chr_ram_size: 0,
            console_type: ConsoleType::Nes,
            title: "Test ROM".to_string(),
        }
    }
}
