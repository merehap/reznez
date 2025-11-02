use std::fmt;

use log::warn;

use crate::cartridge::cartridge_metadata::{CartridgeMetadata, CartridgeMetadataBuilder, ConsoleType, ExpansionDevice, TimingMode, VsHardwareType, VsPpuType};
use crate::mapper::NameTableMirroring;

use crate::util::unit::KIBIBYTE;

#[derive(Clone, Debug, Default)]
pub struct ResolvedMetadata {
    pub mapper_number: u16,
    pub submapper_number: Option<u8>,

    // FIXME: This should no longer be Optional.
    pub name_table_mirroring: Option<NameTableMirroring>,
    pub has_persistent_memory: bool,

    pub full_hash: u32,
    pub prg_rom_hash: u32,
    pub chr_rom_hash: u32,

    pub prg_rom_size: u32,
    pub prg_work_ram_size: u32,
    pub prg_save_ram_size: u32,

    pub chr_rom_size: u32,
    pub chr_work_ram_size: u32,
    pub chr_save_ram_size: u32,

    pub console_type: ConsoleType,
    pub region_timing_mode: TimingMode,
    pub miscellaneous_rom_count: u8,
    pub default_expansion_device: ExpansionDevice,
    pub vs: Option<Vs>,
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
    pub hard_coded_overrides: CartridgeMetadata,
    pub cartridge: CartridgeMetadata,
    pub database: CartridgeMetadata,
    pub database_extension: CartridgeMetadata,
    pub layout_supports_prg_ram: bool,
    // TODO: Add user overrides.
}

impl MetadataResolver {
    pub fn resolve(&self) -> ResolvedMetadata {
        let all_metadata = [&self.hard_coded_overrides, &self.cartridge, &self.database, &self.database_extension, &self.defaults()];

        let mut vs = None;
        if let (Some(hardware_type), Some(ppu_type))
                = (resolve_field(&all_metadata, |m| m.vs_hardware_type()), resolve_field(&all_metadata, |m| m.vs_ppu_type())) {
            vs = Some(Vs { hardware_type, ppu_type });
        }

        let resolved_metadata = ResolvedMetadata {
            mapper_number: resolve_field(&all_metadata, |m| m.mapper_number()).unwrap(),
            submapper_number: resolve_field(&all_metadata, |m| m.submapper_number()),

            name_table_mirroring: resolve_field(&all_metadata, |m| m.name_table_mirroring()),
            has_persistent_memory: resolve_field(&all_metadata, |m| m.has_persistent_memory()).unwrap(),
            console_type: resolve_field(&all_metadata, |m| m.console_type()).unwrap(),

            // TODO: Verify that all hashes match.
            full_hash: self.cartridge.full_hash().unwrap(),
            prg_rom_hash: self.cartridge.prg_rom_hash().unwrap(),
            chr_rom_hash: self.cartridge.chr_rom_hash().unwrap(),

            // TODO: Verify that all PRG ROM sizes match.
            prg_rom_size: self.cartridge.prg_rom_size().unwrap(),
            prg_work_ram_size: resolve_field(&all_metadata, |m| m.prg_work_ram_size()).unwrap(),
            prg_save_ram_size: resolve_field(&all_metadata, |m| m.prg_save_ram_size()).unwrap(),

            // TODO: Verify that all CHR ROM sizes match.
            chr_rom_size: self.cartridge.chr_rom_size().unwrap(),
            chr_work_ram_size: resolve_field(&all_metadata, |m| m.chr_work_ram_size()).unwrap(),
            chr_save_ram_size: resolve_field(&all_metadata, |m| m.chr_save_ram_size()).unwrap(),

            region_timing_mode: resolve_field(&all_metadata, |m| m.timing_mode()).unwrap(),
            miscellaneous_rom_count: resolve_field(&all_metadata, |m| m.miscellaneous_rom_count()).unwrap(),
            default_expansion_device: resolve_field(&all_metadata, |m| m.default_expansion_device()).unwrap(),
            vs,
        };

        if self.database.name_table_mirroring().is_some() {
            if resolved_metadata.name_table_mirroring != self.database.name_table_mirroring() {
                warn!("NameTableMirroring {} doesn't match the nes20db.xml entry {}. DB full hash: {:X}, DB PRG ROM hash: {:X}",
                    resolved_metadata.name_table_mirroring.unwrap(),
                    self.database.name_table_mirroring().unwrap(),
                    self.database.full_hash().unwrap(),
                    self.database.prg_rom_hash().unwrap(),
                );
            }
        }

        resolved_metadata
    }

    pub fn defaults(&self) -> CartridgeMetadata {
        let all_metadata = [&self.hard_coded_overrides, &self.database_extension, &self.cartridge, &self.database];

        let prg_work_ram_size = if self.layout_supports_prg_ram { 8 * KIBIBYTE } else { 0 };

        let chr_rom_size = self.cartridge.chr_rom_size();
        let chr_work_ram_size = resolve_field(&all_metadata, |m| m.chr_work_ram_size());
        let chr_save_ram_size = resolve_field(&all_metadata, |m| m.chr_save_ram_size());
        let chr_work_ram_size = if matches!(chr_rom_size, None | Some(0)) && chr_work_ram_size.is_none() && chr_save_ram_size.is_none() {
            8 * KIBIBYTE
        } else {
            0
        };

        let mut builder = CartridgeMetadataBuilder::new();
        builder
            .prg_work_ram_size(prg_work_ram_size)
            .prg_save_ram_size(0)
            .chr_work_ram_size(chr_work_ram_size)
            .chr_save_ram_size(0)
            .timing_mode(TimingMode::Ntsc)
            .miscellaneous_rom_count(0)
            .default_expansion_device(ExpansionDevice::StandardNesFamicomControllers);

        let console_type = resolve_field(&all_metadata, |m| m.console_type()).unwrap();
        if console_type == ConsoleType::PlayChoice10 {
            builder
                .vs_hardware_type(VsHardwareType::Unisystem)
                .vs_ppu_type(VsPpuType::Rp2c03Rc2c03);
        }

        builder.build()
    }
}

fn resolve_field<F, T>(all_metadata: &[&CartridgeMetadata], field_extractor: F) -> Option<T>
where F: Fn(&&CartridgeMetadata) -> Option<T> {
    all_metadata.iter().map(field_extractor).find(|v| v.is_some()).flatten()
}

#[derive(Clone, Debug, Default)]
pub struct Vs {
    pub hardware_type: VsHardwareType,
    pub ppu_type: VsPpuType,
}