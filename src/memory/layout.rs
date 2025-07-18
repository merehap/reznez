use std::collections::BTreeSet;
use std::num::NonZeroU8;

use log::warn;

use crate::cartridge::cartridge::Cartridge;
use crate::memory::bank::bank_index::{BankIndex, PrgBankRegisterId, PrgBankRegisters, ChrBankRegisters, MetaRegisterId};
use crate::memory::cpu::prg_layout::PrgLayout;
use crate::memory::cpu::prg_memory::PrgMemory;
use crate::mapper::{MapperParams, ReadWriteStatus};
use crate::memory::ppu::chr_layout::ChrLayout;
use crate::memory::ppu::chr_memory::{AccessOverride, ChrMemory};
use crate::memory::raw_memory::{RawMemory, SaveRam};
use crate::memory::window::{ReadWriteStatusInfo, PrgWindow};
use crate::ppu::name_table::name_table_mirroring::NameTableMirroring;
use crate::util::const_vec::ConstVec;
use crate::util::unit::KIBIBYTE;

use super::bank::bank_index::ChrBankRegisterId;
use super::window::ChrWindow;

#[derive(Clone)]
pub struct Layout {
    prg_rom_max_size: u32,
    prg_layout_index: u8,
    prg_layouts: ConstVec<PrgLayout, 10>,
    prg_rom_outer_bank_layout: OuterBankLayout,

    chr_rom_max_size: u32,
    align_large_chr_windows: bool,
    chr_layout_index: u8,
    chr_layouts: ConstVec<ChrLayout, 10>,
    chr_save_ram_size: u32,
    chr_rom_outer_bank_layout: OuterBankLayout,

    name_table_mirroring_source: NameTableMirroringSource,
    name_table_mirrorings: &'static [NameTableMirroring],

    read_write_statuses: &'static [ReadWriteStatus],

    bank_register_overrides: ConstVec<(PrgBankRegisterId, BankIndex), 5>,
    chr_bank_register_overrides: ConstVec<(ChrBankRegisterId, BankIndex), 5>,
    chr_meta_register_overrides: ConstVec<(MetaRegisterId, ChrBankRegisterId), 5>,
}

impl Layout {
    pub const fn builder() -> LayoutBuilder {
        LayoutBuilder::new()
    }

    pub fn make_mapper_params(self, cartridge: &Cartridge) -> MapperParams {
        let prg_rom_size = cartridge.prg_rom().size();
        assert!(prg_rom_size <= self.prg_rom_max_size,
            "PRG ROM size of {}KiB is too large for this mapper.", prg_rom_size / KIBIBYTE);
        let chr_rom_size = cartridge.chr_rom().size();
        assert!(chr_rom_size <= self.chr_rom_max_size,
            "CHR ROM size of {}KiB is too large for this mapper.", chr_rom_size / KIBIBYTE);

        let mut chr_access_override = cartridge.chr_access_override();
        let chr_ram = if self.chr_save_ram_size > 0 {
            match cartridge.chr_ram().size() {
                0 => {
                    if chr_access_override.is_some() {
                        warn!("Removing CHR access override because mapper explicitly set CHR Save RAM.");
                        chr_access_override = None;
                    }

                    RawMemory::new(self.chr_save_ram_size)
                }
                size if size == self.chr_save_ram_size => RawMemory::new(self.chr_save_ram_size),
                _ => panic!("CHR SAVE RAM size from cartridge did not match mapper override value."),
            }
        } else {
            cartridge.chr_ram()
        };

        let mut prg_bank_registers = PrgBankRegisters::new();
        for (register_id, bank_index) in self.bank_register_overrides.as_iter() {
            prg_bank_registers.set(register_id, bank_index);
        }

        let mut chr_bank_registers = ChrBankRegisters::new();
        for (register_id, bank_index) in self.chr_bank_register_overrides.as_iter() {
            chr_bank_registers.set(register_id, bank_index);
        }

        for (meta_id, register_id) in self.chr_meta_register_overrides.as_iter() {
            chr_bank_registers.set_meta_chr(meta_id, register_id);
        }

        let mut prg_layouts: Vec<_> = self.prg_layouts.as_iter().collect();
        if cartridge.prg_rom_forced() {
            for layout in &mut prg_layouts {
                *layout = layout.force_rom()
            }
        }

        let mut prg_memory = PrgMemory::new(
            prg_layouts,
            self.prg_layout_index,
            cartridge.prg_rom().clone(),
            self.prg_rom_outer_bank_layout.outer_bank_count(prg_rom_size),
            RawMemory::new(cartridge.prg_work_ram_size()),
            SaveRam::open(&cartridge.path().to_prg_save_ram_file_path(), cartridge.prg_save_ram_size(), cartridge.allow_saving()),
            prg_bank_registers,
        );

        let name_table_mirroring = match self.name_table_mirroring_source {
            NameTableMirroringSource::Direct(mirroring) => mirroring,
            NameTableMirroringSource::Cartridge => cartridge.name_table_mirroring()
                .expect("This mapper must define what Four Screen mirroring is."),
        };

        let mut chr_layouts: Vec<_> = self.chr_layouts.as_iter().collect();
        match chr_access_override {
            None => {}
            Some(AccessOverride::ForceRom) => {
                for layout in &mut chr_layouts {
                    *layout = layout.force_rom()
                }
            }
            Some(AccessOverride::ForceRam) => {
                for layout in &mut chr_layouts {
                    *layout = layout.force_ram()
                }
            }
        }

        let mut chr_memory = ChrMemory::new(
            chr_layouts,
            self.chr_layout_index,
            self.align_large_chr_windows,
            self.chr_rom_outer_bank_layout.outer_bank_count(chr_rom_size),
            cartridge.chr_rom().clone(),
            chr_ram,
            name_table_mirroring,
            chr_bank_registers,
        );

        let mut ram_not_present = BTreeSet::new();
        if cartridge.prg_work_ram_size() == 0 && cartridge.prg_save_ram_size() == 0 {
            for status_info in prg_memory.read_write_status_infos() {
                match status_info {
                    ReadWriteStatusInfo::Absent | ReadWriteStatusInfo::MapperCustom { .. } => { /* Do nothing. */ }
                    ReadWriteStatusInfo::PossiblyPresent { register_id, status_on_absent } => {
                        prg_memory.set_read_write_status(register_id, status_on_absent);
                        ram_not_present.insert(register_id);
                    }
                }
            }
        }

        if cartridge.chr_work_ram_size() == 0 && cartridge.chr_save_ram_size() == 0 {
            for status_info in chr_memory.read_write_status_infos() {
                match status_info {
                    ReadWriteStatusInfo::Absent | ReadWriteStatusInfo::MapperCustom { .. } => { /* Do nothing. */ }
                    ReadWriteStatusInfo::PossiblyPresent { register_id, status_on_absent } => {
                        chr_memory.set_read_write_status(register_id, status_on_absent);
                        ram_not_present.insert(register_id);
                    }
                }
            }
        }

        MapperParams {
            prg_memory,
            chr_memory,
            name_table_mirrorings: self.name_table_mirrorings,
            read_write_statuses: self.read_write_statuses,
            ram_not_present,
            irq_pending: false,
        }
    }

    pub const fn into_builder(self) -> LayoutBuilder {
        LayoutBuilder {
            prg_rom_max_size: Some(self.prg_rom_max_size),
            prg_layout_index: self.prg_layout_index,
            prg_layouts: self.prg_layouts,
            prg_rom_outer_bank_layout: Some(self.prg_rom_outer_bank_layout),

            chr_rom_max_size: Some(self.chr_rom_max_size),
            align_large_chr_windows: self.align_large_chr_windows,
            chr_layout_index: self.chr_layout_index,
            chr_layouts: self.chr_layouts,
            chr_rom_outer_bank_layout: Some(self.chr_rom_outer_bank_layout),
            chr_save_ram_size: self.chr_save_ram_size,

            name_table_mirroring_source: self.name_table_mirroring_source,
            name_table_mirrorings: self.name_table_mirrorings,

            read_write_statuses: self.read_write_statuses,

            bank_register_overrides: self.bank_register_overrides,
            chr_bank_register_overrides: self.chr_bank_register_overrides,
            chr_meta_register_overrides: self.chr_meta_register_overrides,
        }
    }

    pub const fn into_builder_with_prg_layouts_cleared(self) -> LayoutBuilder {
        assert!(self.prg_layout_index == 0, "PRG Layout Index must be zero.");

        let mut builder = self.into_builder();
        builder.prg_layouts = ConstVec::new();
        builder
    }

    pub const fn into_builder_with_chr_layouts_cleared(self) -> LayoutBuilder {
        assert!(self.chr_layout_index == 0, "CHR Layout Index must be zero.");

        let mut builder = self.into_builder();
        builder.chr_layouts = ConstVec::new();
        builder
    }
}

#[derive(Clone, Copy)]
pub struct LayoutBuilder {
    prg_rom_max_size: Option<u32>,
    prg_layouts: ConstVec<PrgLayout, 10>,
    prg_layout_index: u8,
    prg_rom_outer_bank_layout: Option<OuterBankLayout>,

    chr_rom_max_size: Option<u32>,
    chr_layouts: ConstVec<ChrLayout, 10>,
    chr_layout_index: u8,
    chr_rom_outer_bank_layout:  Option<OuterBankLayout>,
    align_large_chr_windows: bool,
    chr_save_ram_size: u32,

    name_table_mirroring_source: NameTableMirroringSource,
    name_table_mirrorings: &'static [NameTableMirroring],

    read_write_statuses: &'static [ReadWriteStatus],

    bank_register_overrides: ConstVec<(PrgBankRegisterId, BankIndex), 5>,
    chr_bank_register_overrides: ConstVec<(ChrBankRegisterId, BankIndex), 5>,
    chr_meta_register_overrides: ConstVec<(MetaRegisterId, ChrBankRegisterId), 5>,
}

impl LayoutBuilder {
    const fn new() -> LayoutBuilder {
        LayoutBuilder {
            prg_rom_max_size: None,
            prg_layout_index: 0,
            prg_layouts: ConstVec::new(),
            prg_rom_outer_bank_layout: None,

            chr_rom_max_size: None,
            align_large_chr_windows: true,
            chr_layout_index: 0,
            chr_layouts: ConstVec::new(),
            chr_rom_outer_bank_layout: None,
            chr_save_ram_size: 0,

            name_table_mirroring_source: NameTableMirroringSource::Cartridge,
            name_table_mirrorings: &[],

            read_write_statuses: &[],

            bank_register_overrides: ConstVec::new(),
            chr_bank_register_overrides: ConstVec::new(),
            chr_meta_register_overrides: ConstVec::new(),
        }
    }

    pub const fn prg_rom_max_size(&mut self, value: u32) -> &mut LayoutBuilder {
        self.prg_rom_max_size = Some(value);
        self
    }

    pub const fn prg_layout_index(&mut self, value: u8) -> &mut LayoutBuilder {
        self.prg_layout_index = value;
        self
    }

    pub const fn prg_layout(&mut self, windows: &'static [PrgWindow]) -> &mut LayoutBuilder {
        self.prg_layouts.push(PrgLayout::new(windows));
        self
    }

    pub const fn prg_outer_bank_count(&mut self, count: u8) -> &mut LayoutBuilder {
        self.prg_rom_outer_bank_layout = Some(OuterBankLayout::ExactCount(NonZeroU8::new(count).unwrap()));
        self
    }

    pub const fn prg_rom_outer_bank_size(&mut self, size: u32) -> &mut LayoutBuilder {
        self.prg_rom_outer_bank_layout = Some(OuterBankLayout::Size(size));
        self
    }

    pub const fn chr_rom_max_size(&mut self, value: u32) -> &mut LayoutBuilder {
        self.chr_rom_max_size = Some(value);
        self
    }

    pub const fn chr_layout(&mut self, windows: &'static [ChrWindow]) -> &mut LayoutBuilder {
        self.chr_layouts.push(ChrLayout::new(windows));
        self
    }

    pub const fn chr_layout_index(&mut self, value: u8) -> &mut LayoutBuilder {
        self.chr_layout_index = value;
        self
    }

    pub const fn do_not_align_large_chr_windows(&mut self) -> &mut LayoutBuilder {
        self.align_large_chr_windows = false;
        self
    }

    pub const fn chr_save_ram_size(&mut self, value: u32) -> &mut LayoutBuilder {
        self.chr_save_ram_size = value;
        self
    }

    pub const fn chr_rom_outer_bank_size(&mut self, size: u32) -> &mut LayoutBuilder {
        self.chr_rom_outer_bank_layout = Some(OuterBankLayout::Size(size));
        self
    }

    pub const fn initial_name_table_mirroring(
        &mut self,
        value: NameTableMirroring,
    ) -> &mut LayoutBuilder {
        self.name_table_mirroring_source = NameTableMirroringSource::Direct(value);
        self
    }

    pub const fn name_table_mirrorings(
        &mut self,
        value: &'static [NameTableMirroring],
    ) -> &mut LayoutBuilder {
        self.name_table_mirrorings = value;
        self
    }

    pub const fn read_write_statuses(
        &mut self,
        value: &'static [ReadWriteStatus],
    ) -> &mut LayoutBuilder {
        self.read_write_statuses = value;
        self
    }

    pub const fn override_prg_bank_register(
        &mut self,
        id: PrgBankRegisterId,
        bank_index: i16,
    ) -> &mut LayoutBuilder {
        self.bank_register_overrides.push((id, BankIndex::from_i16(bank_index)));
        self
    }

    pub const fn override_chr_bank_register(
        &mut self,
        id: ChrBankRegisterId,
        bank_index: i16,
    ) -> &mut LayoutBuilder {
        self.chr_bank_register_overrides.push((id, BankIndex::from_i16(bank_index)));
        self
    }

    pub const fn override_chr_meta_register(
        &mut self,
        meta_id: MetaRegisterId,
        id: ChrBankRegisterId,
    ) -> &mut LayoutBuilder {
        self.chr_meta_register_overrides.push((meta_id, id));
        self
    }

    pub const fn build(self) -> Layout {
        assert!(!self.prg_layouts.is_empty());
        assert!(!self.chr_layouts.is_empty());

        let mut prg_rom_outer_bank_layout = OuterBankLayout::SINGLE_BANK;
        if let Some(layout) = self.prg_rom_outer_bank_layout {
            prg_rom_outer_bank_layout = layout;
        }

        let mut chr_rom_outer_bank_layout = OuterBankLayout::SINGLE_BANK;
        if let Some(layout) = self.chr_rom_outer_bank_layout {
            chr_rom_outer_bank_layout = layout;
        }

        Layout {
            prg_rom_max_size: self.prg_rom_max_size.unwrap(),
            prg_layouts: self.prg_layouts,
            prg_layout_index: self.prg_layout_index,
            prg_rom_outer_bank_layout,

            chr_rom_max_size: self.chr_rom_max_size.unwrap(),
            chr_layouts: self.chr_layouts,
            chr_layout_index: self.chr_layout_index,
            align_large_chr_windows: self.align_large_chr_windows,
            chr_save_ram_size: self.chr_save_ram_size,
            chr_rom_outer_bank_layout,

            name_table_mirroring_source: self.name_table_mirroring_source,
            name_table_mirrorings: self.name_table_mirrorings,

            read_write_statuses: self.read_write_statuses,

            bank_register_overrides: self.bank_register_overrides,
            chr_bank_register_overrides: self.chr_bank_register_overrides,
            chr_meta_register_overrides: self.chr_meta_register_overrides,
        }
    }
}

#[derive(Clone, Copy)]
pub enum NameTableMirroringSource {
    Direct(NameTableMirroring),
    Cartridge,
}

impl NameTableMirroring {
    pub const fn to_source(self) -> NameTableMirroringSource {
        NameTableMirroringSource::Direct(self)
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
enum OuterBankLayout {
    ExactCount(NonZeroU8),
    Size(u32),
}

impl OuterBankLayout {
    const SINGLE_BANK: Self = Self::ExactCount(NonZeroU8::new(1).unwrap());

    fn outer_bank_count(self, memory_size: u32) -> NonZeroU8 {
        match self {
            Self::ExactCount(count) => count,
            Self::Size(size) => {
                let count = if memory_size < size {
                    1
                } else {
                    assert_eq!(memory_size % size, 0);
                    (memory_size / size).try_into().unwrap()
                };
                NonZeroU8::new(count).unwrap()
            }
        }
    }
}