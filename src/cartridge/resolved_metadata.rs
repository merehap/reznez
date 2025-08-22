use std::fmt;

use crate::cartridge::cartridge_metadata::{CartridgeMetadata, CartridgeMetadataBuilder, ConsoleType};
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
    pub mapper: CartridgeMetadata,
    pub hard_coded_overrides: CartridgeMetadata,
    pub cartridge: CartridgeMetadata,
    pub database: CartridgeMetadata,
    pub database_extension: CartridgeMetadata,
    pub layout_has_prg_ram: bool,
    // TODO: Add user overrides.
}

impl MetadataResolver {
    pub fn resolve(&self) -> ResolvedMetadata {
        let all_metadata = [&self.mapper, &self.hard_coded_overrides, &self.database_extension, &self.cartridge, &self.database, &self.defaults()];

        ResolvedMetadata {
            mapper_number: resolve_field(&all_metadata, |m| m.mapper_number()).unwrap(),
            submapper_number: resolve_field(&all_metadata, |m| m.submapper_number()),

            name_table_mirroring: resolve_field(&all_metadata, |m| m.name_table_mirroring()).expect("This mapper must define what Four Screen mirroring is."),
            has_persistent_memory: resolve_field(&all_metadata, |m| m.has_persistent_memory()).unwrap(),
            console_type: resolve_field(&all_metadata, |m| m.console_type()).unwrap(),

            // TODO: Verify that all hashes match.
            full_hash: self.cartridge.full_hash().unwrap(),
            prg_rom_hash: self.cartridge.prg_rom_hash().unwrap(),

            // TODO: Verify that all PRG ROM sizes match.
            prg_rom_size: self.cartridge.prg_rom_size().unwrap(),
            prg_work_ram_size: resolve_field(&all_metadata, |m| m.prg_work_ram_size()).unwrap(),
            prg_save_ram_size: resolve_field(&all_metadata, |m| m.prg_save_ram_size()).unwrap(),

            // TODO: Verify that all CHR ROM sizes match.
            chr_rom_size: self.cartridge.chr_rom_size().unwrap(),
            chr_work_ram_size: resolve_field(&all_metadata, |m| m.chr_work_ram_size()).unwrap(),
            chr_save_ram_size: resolve_field(&all_metadata, |m| m.chr_save_ram_size()).unwrap(),
        }
    }

    pub fn defaults(&self) -> CartridgeMetadata {
        let all_metadata = [&self.mapper, &self.database_extension, &self.cartridge, &self.database];

        let prg_work_ram_size = if self.layout_has_prg_ram { 8 * KIBIBYTE } else { 0 };

        let chr_rom_size = self.cartridge.chr_rom_size();
        let chr_work_ram_size = resolve_field(&all_metadata, |m| m.chr_work_ram_size());
        let chr_save_ram_size = resolve_field(&all_metadata, |m| m.chr_save_ram_size());
        let chr_work_ram_size = if matches!(chr_rom_size, None | Some(0)) && chr_work_ram_size.is_none() && chr_save_ram_size.is_none() {
            8 * KIBIBYTE
        } else {
            0
        };

        CartridgeMetadataBuilder::new()
            .console_type(ConsoleType::Nes)
            .prg_work_ram_size(prg_work_ram_size)
            .prg_save_ram_size(0)
            .chr_work_ram_size(chr_work_ram_size)
            .chr_save_ram_size(0)
            .build()
    }
}

fn resolve_field<F, T>(all_metadata: &[&CartridgeMetadata], field_extractor: F) -> Option<T>
where F: Fn(&&CartridgeMetadata) -> Option<T> {
    all_metadata.iter().map(field_extractor).find(|v| v.is_some()).flatten()
}