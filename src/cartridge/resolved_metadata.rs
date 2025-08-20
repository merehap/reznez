use std::fmt;

use crate::cartridge::cartridge_metadata::{CartridgeMetadata, ConsoleType};
use crate::mapper::NameTableMirroring;

use crate::util::unit::KIBIBYTE;

#[derive(Clone, Debug, Default)]
pub struct ResolvedMetadata {
    pub mapper_number: u16,
    pub submapper_number: Option<u8>,

    pub name_table_mirroring: NameTableMirroring,
    pub has_persistent_memory: bool,
    pub console_type: ConsoleType,

    pub full_hash: u32,
    pub prg_rom_hash: u32,

    pub prg_rom_size: u32,
    pub prg_work_ram_size: u32,
    pub prg_save_ram_size: u32,

    pub chr_rom_size: u32,
    pub chr_work_ram_size: u32,
    pub chr_save_ram_size: u32,
}

impl fmt::Display for ResolvedMetadata {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Mapper: {}", self.mapper_number)?;
        if let Some(submapper_number) = self.submapper_number {
            write!(f, " (Submapper: {submapper_number})")?;
        }

        writeln!(f)?;
        writeln!(f, "PRG ROM: {:4}KiB, WorkRAM: {:4}KiB, SaveRAM: {:4}KiB",
            self.prg_rom_size / KIBIBYTE,
            self.prg_work_ram_size / KIBIBYTE,
            self.prg_save_ram_size / KIBIBYTE,
        )?;
        writeln!(f, "CHR ROM: {:4}KiB, WorkRAM: {:4}KiB, SaveRAM: {:4}KiB",
            self.chr_rom_size / KIBIBYTE,
            self.chr_work_ram_size / KIBIBYTE,
            self.chr_save_ram_size / KIBIBYTE,
        )?;

        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct MetadataResolver {
    pub cartridge: CartridgeMetadata,
    pub database: CartridgeMetadata,
    // TODO: Split this into database_override_source.
    pub database_extension: CartridgeMetadata,
    pub mapper: CartridgeMetadata,
    pub default: CartridgeMetadata,
    // TODO: Add user overrides.
}

impl MetadataResolver {
    pub fn resolve(&self) -> ResolvedMetadata {
        let all_metadata = [&self.cartridge, &self.database, &self.database_extension, &self.mapper, &self.default];

        let submapper_number = if self.database_extension.submapper_number().is_some() {
            // FIXME: Remove this hack. Database extension needs to be split into DB ext and overrides.
            self.database_extension.submapper_number()
        } else {
            all_metadata.iter().map(|m| m.submapper_number()).find(|s| s.is_some()).flatten()
        };

        let name_table_mirroring = self.mapper.name_table_mirroring()
            .or(all_metadata.iter()
                   .map(|m| m.name_table_mirroring())
                   .find(|s| s.is_some())
                   .flatten()
            ).expect("This mapper must define what Four Screen mirroring is.");

        let chr_rom_size = self.cartridge.chr_rom_size().unwrap();
        let mut chr_work_ram_size = resolve_field(&all_metadata, |m| m.chr_work_ram_size());
        let chr_save_ram_size = resolve_field(&all_metadata, |m| m.chr_save_ram_size());
        if chr_rom_size == 0 && chr_work_ram_size == 0 && chr_save_ram_size == 0 {
            chr_work_ram_size = 8 * KIBIBYTE;
        }

        ResolvedMetadata {
            mapper_number: resolve_field(&all_metadata, |m| m.mapper_number()),
            submapper_number,

            name_table_mirroring,
            has_persistent_memory: resolve_field(&all_metadata, |m| m.has_persistent_memory()),
            console_type: resolve_field(&all_metadata, |m| m.console_type()),

            // TODO: Verify that all hashes match.
            full_hash: self.cartridge.full_hash().unwrap(),
            prg_rom_hash: self.cartridge.prg_rom_hash().unwrap(),

            // TODO: Verify that all PRG ROM sizes match.
            prg_rom_size: self.cartridge.prg_rom_size().unwrap(),
            prg_work_ram_size: resolve_field(&all_metadata, |m| m.prg_work_ram_size()),
            prg_save_ram_size: resolve_field(&all_metadata, |m| m.prg_save_ram_size()),

            // TODO: Verify that all CHR ROM sizes match.
            chr_rom_size,
            chr_work_ram_size,
            chr_save_ram_size,
        }
    }
}

fn resolve_field<F, T>(all_metadata: &[&CartridgeMetadata; 5], field_extractor: F) -> T
where F: Fn(&&CartridgeMetadata) -> Option<T> {
    all_metadata.iter().map(field_extractor).find(|v| v.is_some()).flatten().unwrap()
}