use std::fmt;

use splitbits::{splitbits, splitbits_named};

use crate::ppu::name_table::name_table_mirroring::NameTableMirroring;
use crate::util::unit::KIBIBYTE;

pub const PRG_ROM_CHUNK_LENGTH: usize = 16 * KIBIBYTE as usize;
pub const CHR_ROM_CHUNK_LENGTH: usize = 8 * KIBIBYTE as usize;
const INES_HEADER_CONSTANT: &[u8] = &[b'N', b'E', b'S', 0x1A];

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct CartridgeHeader {
    mapper_number: u16,
    name_table_mirroring: Option<NameTableMirroring>,
    has_persistent_memory: bool,
    console_type: ConsoleType,

    prg_rom_size: u32,
    chr_rom_size: Option<u32>,

    submapper_number: Option<u8>,

    prg_work: Option<u32>,
    prg_save: Option<u32>,
    chr_work: Option<u32>,
    chr_save: Option<u32>,
}

impl CartridgeHeader {
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
        let mut submapper_number = None;
        let mut prg_work = None;
        let mut prg_save = None;
        let mut chr_work = None;
        let mut chr_save = None;
        if ines2_present {
            mapper_number |= u16::from(header[8] & 0b1111) << 8;
            submapper_number = Some(header[8] >> 4);
            let prg_sizes = splitbits!(min=u32, header[10], "sssswwww");
            prg_work = Some(if prg_sizes.w > 0 { 64 << prg_sizes.w } else { 0 });
            prg_save = Some(if prg_sizes.s > 0 { 64 << prg_sizes.s } else { 0 });

            let chr_sizes = splitbits!(min=u32, header[11], "sssswwww");
            chr_work = Some(if chr_sizes.w > 0 { 64 << chr_sizes.w } else { 0 });
            chr_save = Some(if chr_sizes.s > 0 { 64 << chr_sizes.s } else { 0 });
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

        Ok(CartridgeHeader {
            mapper_number,
            submapper_number,
            name_table_mirroring,
            has_persistent_memory,
            console_type: ConsoleType::Nes,
            prg_rom_size: prg_rom_chunk_count * PRG_ROM_CHUNK_LENGTH as u32,
            chr_rom_size: Some(chr_rom_chunk_count * CHR_ROM_CHUNK_LENGTH as u32),

            prg_work,
            prg_save,
            chr_work,
            chr_save,
        })
    }

    pub fn mapper_number(&self) -> Option<u16> {
        Some(self.mapper_number)
    }

    pub fn submapper_number(&self) -> Option<u8> {
        self.submapper_number
    }

    pub fn prg_rom_size(&self) -> Option<u32> {
        Some(self.prg_rom_size)
    }

    pub fn prg_work_ram_size(&self) -> Option<u32> {
        self.prg_work
    }

    pub fn prg_save_ram_size(&self) -> Option<u32> {
        self.prg_save
    }

    pub fn chr_rom_size(&self) -> Option<u32> {
        self.chr_rom_size
    }

    pub fn chr_work_ram_size(&self) -> Option<u32> {
        self.chr_work
    }

    pub fn chr_save_ram_size(&self) -> Option<u32> {
        self.chr_save
    }

    // FIXME: This returns None if there is no mirroring specified OR if the cartridge specifies FourScreen.
    pub fn name_table_mirroring(&self) -> Option<NameTableMirroring> {
        self.name_table_mirroring
    }

    pub fn console_type(&self) -> Option<ConsoleType> {
        Some(self.console_type)
    }

    pub fn chr_present(&self) -> bool {
        if let Some(chr_rom) = self.chr_rom_size && chr_rom > 0 {
            return true;
        }

        if let Some(chr_work) = self.chr_work && chr_work > 0 {
            return true;
        }

        if let Some(chr_save) = self.chr_save && chr_save > 0 {
            return true;
        }

        false
    }
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
pub enum ConsoleType {
    Nes,
    VsUnisystem,
    PlayChoice10,
    Extended,
}

impl fmt::Display for ConsoleType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self.clone() {
            ConsoleType::Nes => "NES",
            ConsoleType::VsUnisystem => "VS Unisystem",
            ConsoleType::PlayChoice10 => "Play Choice 10",
            ConsoleType::Extended => "Extended",
        };

        write!(f, "{text}")
    }
}
