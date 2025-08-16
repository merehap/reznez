use std::fmt;

use splitbits::{combinebits, splitbits, splitbits_named};

use crate::ppu::name_table::name_table_mirroring::NameTableMirroring;
use crate::util::unit::KIBIBYTE;

pub const PRG_ROM_CHUNK_LENGTH: usize = 16 * KIBIBYTE as usize;
pub const CHR_ROM_CHUNK_LENGTH: usize = 8 * KIBIBYTE as usize;
const INES_HEADER_CONSTANT: &[u8] = &[b'N', b'E', b'S', 0x1A];

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct CartridgeHeader {
    mapper_number: Option<u16>,
    submapper_number: Option<u8>,

    name_table_mirroring: Option<NameTableMirroring>,
    has_persistent_memory: Option<bool>,
    console_type: Option<ConsoleType>,

    full_hash: Option<u32>,
    prg_rom_hash: Option<u32>,

    prg_rom_size: Option<u32>,
    prg_work_ram_size: Option<u32>,
    prg_save_ram_size: Option<u32>,

    chr_rom_size: Option<u32>,
    chr_work_ram_size: Option<u32>,
    chr_save_ram_size: Option<u32>,
}

impl CartridgeHeader {
    pub fn parse(header: [u8; 16], full_hash: u32) -> Result<CartridgeHeader, String> {
        let mut builder = CartridgeHeaderBuilder::new();
        builder.full_hash(full_hash);

        if &header[0..4] != INES_HEADER_CONSTANT {
            return Err(format!(
                "Cannot load non-iNES ROM. Found {:?} but need {:?}.",
                &header[0..4],
                INES_HEADER_CONSTANT,
            ));
        }

        builder
            .prg_rom_size(header[4] as u32 * PRG_ROM_CHUNK_LENGTH as u32)
            .chr_rom_size(header[5] as u32 * CHR_ROM_CHUNK_LENGTH as u32);

        let (low_mapper_number, four_screen, trainer_enabled, has_persistent_memory, vertical_mirroring) =
            splitbits_named!(header[6], "llllftpv");
        let (mid_mapper_number, ines2, play_choice_enabled, vs_unisystem_enabled) =
            splitbits_named!(header[7], "mmmmiipv");

        builder.has_persistent_memory(has_persistent_memory);
        let ines2_present = ines2 == 0b10;
        if trainer_enabled {
            return Err("Trainer isn't implemented yet.".to_string());
        }

        let mut high_mapper_number = 0b0000;
        if ines2_present {
            high_mapper_number = header[8] & 0b1111;
            builder.submapper_number(header[8] >> 4);
            let prg_sizes = splitbits!(min=u32, header[10], "sssswwww");
            let prg_work = if prg_sizes.w > 0 { 64 << prg_sizes.w } else { 0 };
            builder.prg_work_ram_size(prg_work);
            let prg_save = if prg_sizes.s > 0 { 64 << prg_sizes.s } else { 0 };
            builder.prg_save_ram_size(prg_save);

            let chr_sizes = splitbits!(min=u32, header[11], "sssswwww");
            let chr_work = if chr_sizes.w > 0 { 64 << chr_sizes.w } else { 0 };
            builder.chr_work_ram_size(chr_work);
            let chr_save = if chr_sizes.s > 0 { 64 << chr_sizes.s } else { 0 };
            builder.chr_save_ram_size(chr_save);
        }

        let mapper_number = combinebits!(high_mapper_number, mid_mapper_number, low_mapper_number, "0000uuuummmmllll");
        builder.mapper_number(mapper_number);

        if play_choice_enabled {
            return Err("PlayChoice isn't implemented yet.".to_string());
        }

        if vs_unisystem_enabled {
            return Err("VS Unisystem isn't implemented yet.".to_string());
        }

        if four_screen {
            // Four screen mirroring isn't a real mirroring, the mapper will have to define what it means.
        } else if vertical_mirroring {
            builder.name_table_mirroring(NameTableMirroring::VERTICAL);
        } else {
            builder.name_table_mirroring(NameTableMirroring::HORIZONTAL);
        };

        Ok(builder.build())
    }

    pub fn defaults() -> Self {
        Self {
            console_type: Some(ConsoleType::Nes),
            chr_work_ram_size: Some(8 * KIBIBYTE),
            chr_save_ram_size: Some(0),

            mapper_number: None,
            submapper_number: None,
            name_table_mirroring: None,
            has_persistent_memory: None,
            full_hash: None,
            prg_rom_hash: None,
            prg_rom_size: None,
            prg_work_ram_size: None,
            prg_save_ram_size: None,
            chr_rom_size: None,
        }
    }

    pub fn mapper_number(&self) -> Option<u16> {
        self.mapper_number
    }

    pub fn submapper_number(&self) -> Option<u8> {
        self.submapper_number
    }

    pub fn prg_rom_size(&self) -> Option<u32> {
        self.prg_rom_size
    }

    pub fn prg_work_ram_size(&self) -> Option<u32> {
        self.prg_work_ram_size
    }

    pub fn prg_save_ram_size(&self) -> Option<u32> {
        self.prg_save_ram_size
    }

    pub fn chr_rom_size(&self) -> Option<u32> {
        self.chr_rom_size
    }

    pub fn chr_work_ram_size(&self) -> Option<u32> {
        self.chr_work_ram_size
    }

    pub fn chr_save_ram_size(&self) -> Option<u32> {
        self.chr_save_ram_size
    }

    // FIXME: This returns None if there is no mirroring specified OR if the cartridge specifies FourScreen.
    pub fn name_table_mirroring(&self) -> Option<NameTableMirroring> {
        self.name_table_mirroring
    }

    pub fn console_type(&self) -> Option<ConsoleType> {
        self.console_type
    }

    pub fn chr_present(&self) -> bool {
        if let Some(chr_rom) = self.chr_rom_size && chr_rom > 0 {
            return true;
        }

        if let Some(chr_work) = self.chr_work_ram_size && chr_work > 0 {
            return true;
        }

        if let Some(chr_save) = self.chr_save_ram_size && chr_save > 0 {
            return true;
        }

        false
    }

    pub fn set_prg_rom_hash(&mut self, prg_rom_hash: u32) {
        self.prg_rom_hash = Some(prg_rom_hash);
    }

    pub fn set_submapper_number(&mut self, submapper_number: u8) {
        self.submapper_number = Some(submapper_number);
    }

    pub fn set_console_type(&mut self, console_type: ConsoleType) {
        self.console_type = Some(console_type);
    }

    pub const fn into_builder(self) -> CartridgeHeaderBuilder {
        CartridgeHeaderBuilder {
            mapper_number: self.mapper_number,
            submapper_number: self.submapper_number,

            name_table_mirroring: self.name_table_mirroring,
            has_persistent_memory: self.has_persistent_memory,
            console_type: self.console_type,

            full_hash: self.full_hash,
            prg_rom_hash: self.prg_rom_hash,

            prg_rom_size: self.prg_rom_size,
            prg_work_ram_size: self.prg_work_ram_size,
            prg_save_ram_size: self.prg_save_ram_size,

            chr_rom_size: self.chr_rom_size,
            chr_work_ram_size: self.chr_work_ram_size,
            chr_save_ram_size: self.chr_save_ram_size,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CartridgeHeaderBuilder {
    mapper_number: Option<u16>,
    submapper_number: Option<u8>,

    name_table_mirroring: Option<NameTableMirroring>,
    has_persistent_memory: Option<bool>,
    console_type: Option<ConsoleType>,

    full_hash: Option<u32>,
    prg_rom_hash: Option<u32>,

    prg_rom_size: Option<u32>,
    prg_work_ram_size: Option<u32>,
    prg_save_ram_size: Option<u32>,

    chr_rom_size: Option<u32>,
    chr_work_ram_size: Option<u32>,
    chr_save_ram_size: Option<u32>,
}

impl CartridgeHeaderBuilder {
    pub const fn new() -> Self {
        Self {
            mapper_number: None,
            submapper_number: None,

            name_table_mirroring: None,
            has_persistent_memory: None,
            console_type: None,

            full_hash: None,
            prg_rom_hash: None,

            prg_rom_size: None,
            prg_work_ram_size: None,
            prg_save_ram_size: None,

            chr_rom_size: None,
            chr_work_ram_size: None,
            chr_save_ram_size: None,
        }
    }

    pub const fn mapper_number(&mut self, mapper_number: u16) -> &mut Self {
        self.mapper_number = Some(mapper_number);
        self
    }

    pub const fn submapper_number(&mut self, submapper_number: u8) -> &mut Self {
        self.submapper_number = Some(submapper_number);
        self
    }

    pub const fn name_table_mirroring(&mut self, name_table_mirroring: NameTableMirroring) -> &mut Self {
        self.name_table_mirroring = Some(name_table_mirroring);
        self
    }

    pub const fn has_persistent_memory(&mut self, has_persistent_memory: bool) -> &mut Self {
        self.has_persistent_memory = Some(has_persistent_memory);
        self
    }

    pub const fn console_type(&mut self, console_type: ConsoleType) -> &mut Self {
        self.console_type = Some(console_type);
        self
    }

    pub const fn full_hash(&mut self, full_hash: u32) -> &mut Self {
        self.full_hash = Some(full_hash);
        self
    }

    pub const fn prg_rom_hash(&mut self, prg_rom_hash: u32) -> &mut Self {
        self.prg_rom_hash = Some(prg_rom_hash);
        self
    }

    pub const fn prg_rom_size(&mut self, prg_rom_size: u32) -> &mut Self {
        self.prg_rom_size = Some(prg_rom_size);
        self
    }

    pub const fn prg_work_ram_size(&mut self, prg_work_ram_size: u32) -> &mut Self {
        self.prg_work_ram_size = Some(prg_work_ram_size);
        self
    }

    pub const fn prg_save_ram_size(&mut self, prg_save_ram_size: u32) -> &mut Self {
        self.prg_save_ram_size = Some(prg_save_ram_size);
        self
    }

    pub const fn chr_rom_size(&mut self, chr_rom_size: u32) -> &mut Self {
        self.chr_rom_size = Some(chr_rom_size);
        self
    }

    pub const fn chr_work_ram_size(&mut self, chr_work_ram_size: u32) -> &mut Self {
        self.chr_work_ram_size = Some(chr_work_ram_size);
        self
    }

    pub const fn chr_save_ram_size(&mut self, chr_save_ram_size: u32) -> &mut Self {
        self.chr_save_ram_size = Some(chr_save_ram_size);
        self
    }

    pub const fn build(&mut self) -> CartridgeHeader {
        CartridgeHeader {
            mapper_number: self.mapper_number,
            submapper_number: self.submapper_number,
            name_table_mirroring: self.name_table_mirroring,
            has_persistent_memory: self.has_persistent_memory,
            console_type: self.console_type,
            full_hash: self.full_hash,
            prg_rom_hash: self.prg_rom_hash,
            prg_rom_size: self.prg_rom_size,
            prg_work_ram_size: self.prg_work_ram_size,
            prg_save_ram_size: self.prg_save_ram_size,
            chr_rom_size: self.chr_rom_size,
            chr_work_ram_size: self.chr_work_ram_size,
            chr_save_ram_size: self.chr_save_ram_size,
        }
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
        let text = match self {
            ConsoleType::Nes => "NES",
            ConsoleType::VsUnisystem => "VS Unisystem",
            ConsoleType::PlayChoice10 => "Play Choice 10",
            ConsoleType::Extended => "Extended",
        };

        write!(f, "{text}")
    }
}
