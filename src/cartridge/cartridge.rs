use std::fmt;

use log::error;

use crate::cartridge::header_db::{HeaderDb, Header};
use crate::ppu::name_table::name_table_mirroring::NameTableMirroring;
use crate::util::unit::KIBIBYTE;

const INES_HEADER_CONSTANT: &[u8] = &[0x4E, 0x45, 0x53, 0x1A];
const PRG_ROM_CHUNK_LENGTH: usize = 16 * KIBIBYTE;
const CHR_ROM_CHUNK_LENGTH: usize = 8 * KIBIBYTE;

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

    trainer: Option<[u8; 512]>,

    prg_rom: Vec<u8>,
    prg_ram_size: u32,
    chr_rom: Vec<u8>,
    chr_ram_size: u32,

    console_type: ConsoleType,
    title: String,
}

impl Cartridge {
    #[rustfmt::skip]
    pub fn load(name: String, rom: &[u8], header_db: &HeaderDb) -> Result<Cartridge, String> {
        if &rom[0..4] != INES_HEADER_CONSTANT {
            return Err(format!(
                "Cannot load non-iNES ROM. Found {:?} but need {:?}.",
                &rom[0..4],
                INES_HEADER_CONSTANT,
            ));
        }

        let prg_rom_chunk_count = rom[4] as usize;
        let chr_rom_chunk_count = rom[5] as usize;

        let lower_mapper_number   = rom[6] & 0b1111_0000;
        let four_screen           = rom[6] & 0b0000_1000 != 0;
        let trainer_enabled       = rom[6] & 0b0000_0100 != 0;
        let has_persistent_memory = rom[6] & 0b0000_0010 != 0;
        let vertical_mirroring    = rom[6] & 0b0000_0001 != 0;

        let upper_mapper_number   = rom[7] & 0b1111_0000;
        let ines2_present         = rom[7] & 0b0000_1100 == 0b0000_1100;
        let play_choice_enabled   = rom[7] & 0b0000_0010 != 0;
        let vs_unisystem_enabled  = rom[7] & 0b0000_0001 != 0;

        let ripper_name: String = std::str::from_utf8(&rom[8..15])
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
        let prg_rom_end = prg_rom_start + PRG_ROM_CHUNK_LENGTH * prg_rom_chunk_count;
        let prg_rom = rom.get(prg_rom_start..prg_rom_end)
            .unwrap_or_else(
                || panic!("ROM {name} was too short (claimed to have {prg_rom_chunk_count} PRG chunks)."))
            .to_vec();

        let chr_rom_start = prg_rom_end;
        let mut chr_rom_end = chr_rom_start + CHR_ROM_CHUNK_LENGTH * chr_rom_chunk_count;
        let chr_rom;
        if let Some(chr) = rom.get(chr_rom_start..chr_rom_end) {
            chr_rom = chr.to_vec();
        } else {
            error!("ROM {} claimed to have {} CHR chunks, but the ROM was too short.",
                name, chr_rom_chunk_count);
            chr_rom_end = rom.len();
            chr_rom = rom[chr_rom_start..].to_vec();
        }

        let title_start = chr_rom_end;
        let title = rom[title_start..].to_vec();
        let title_length_is_proper = title.is_empty() || title.len() == 127 || title.len() == 128;
        if !title_length_is_proper {
            return Err(format!("Title must be empty or 127 or 128 bytes, but was {} bytes.", title.len()));
        }

        let title = std::str::from_utf8(&title)
            .map_err(|err| err.to_string())?
            .chars()
            .take_while(|&c| c != '\u{0}')
            .collect();

        let mut cartridge =  Cartridge {
            name,

            mapper_number,
            submapper_number: 0,
            name_table_mirroring,
            has_persistent_memory,
            ripper_name,
            ines2: None,

            trainer: None,
            prg_rom: prg_rom.clone(),
            prg_ram_size: 0,
            chr_rom: chr_rom.clone(),
            chr_ram_size: 0,
            console_type: ConsoleType::Nes,
            title,
        };

        if let Some(Header { prg_rom_size, prg_ram_size, chr_rom_size, chr_ram_size, mapper_number, submapper_number }) =
                header_db.header_from_prg_rom(&prg_rom) {
            assert_eq!(cartridge.mapper_number, mapper_number);
            assert_eq!(prg_rom.len() as u32, prg_rom_size);
            assert_eq!(chr_rom.len() as u32, chr_rom_size);
            cartridge.submapper_number = submapper_number;
            cartridge.prg_ram_size = prg_ram_size;
            cartridge.chr_ram_size = chr_ram_size;
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

    pub fn prg_rom(&self) -> &[u8] {
        &self.prg_rom
    }

    pub fn chr_rom(&self) -> &[u8] {
        &self.chr_rom
    }

    pub fn set_prg_rom_at(&mut self, index: usize, value: u8) {
        self.prg_rom[index] = value;
    }
}

impl fmt::Display for Cartridge {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Mapper: {}", self.mapper_number)?;
        writeln!(f, "Nametable mirroring: {:?}", self.name_table_mirroring)?;
        writeln!(f, "Persistent memory: {}", self.has_persistent_memory)?;
        writeln!(f, "Ripper: {}", self.ripper_name)?;
        writeln!(f, "iNES2 present: {}", self.ines2.is_some())?;

        writeln!(f, "Trainer present: {}", self.trainer.is_some())?;
        writeln!(f, "PRG ROM size: {}KiB", self.prg_rom.len() / KIBIBYTE)?;
        writeln!(f, "CHR ROM size: {}KiB", self.chr_rom.len() / KIBIBYTE)?;
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
    use crate::memory::cpu::cpu_address::CpuAddress;

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
            prg_rom,
            prg_ram_size: 0,
            chr_rom: vec![0x00; CHR_ROM_CHUNK_LENGTH],
            chr_ram_size: 0,
            console_type: ConsoleType::Nes,
            title: "Test ROM".to_string(),
        }
    }

    pub fn cartridge_with_prg_rom(
        mut prg_rom: Vec<u8>,
        nmi_vector: CpuAddress,
        reset_vector: CpuAddress,
        irq_vector: CpuAddress,
    ) -> Cartridge {
        // Filled with NOPs.

        let len = prg_rom.len();
        let (low, high) = nmi_vector.to_low_high();
        prg_rom[len - 6] = low;
        prg_rom[len - 5] = high;
        let (low, high) = reset_vector.to_low_high();
        prg_rom[len - 4] = low;
        prg_rom[len - 3] = high;
        let (low, high) = irq_vector.to_low_high();
        prg_rom[len - 2] = low;
        prg_rom[len - 1] = high;

        Cartridge {
            name: "Test".to_string(),

            mapper_number: 0,
            submapper_number: 0,
            name_table_mirroring: NameTableMirroring::Horizontal,
            has_persistent_memory: false,
            ripper_name: "Test Ripper".to_string(),
            ines2: None,

            trainer: None,
            prg_rom,
            prg_ram_size: 0,
            chr_rom: vec![0x00; CHR_ROM_CHUNK_LENGTH],
            chr_ram_size: 0,
            console_type: ConsoleType::Nes,
            title: "Test ROM".to_string(),
        }
    }
}
