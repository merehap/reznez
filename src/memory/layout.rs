use std::num::NonZeroU8;

use crate::cartridge::cartridge::Cartridge;
use crate::cartridge::resolved_metadata::ResolvedMetadata;
use crate::mapper::ChrSource;
use crate::memory::bank::bank_number::{BankNumber, ChrBankRegisters, MetaRegisterId, PrgBankRegisterId, PrgBankRegisters};
use crate::memory::cpu::prg_layout::PrgLayout;
use crate::memory::cpu::prg_memory::PrgMemory;
use crate::memory::ppu::chr_layout::ChrLayout;
use crate::memory::ppu::chr_memory::ChrMemory;
use crate::memory::raw_memory::{RawMemory, SaveRam};
use crate::memory::window::{PrgWindow, PrgWindowSize};
use crate::ppu::name_table::name_table_mirroring::NameTableMirroring;
use crate::util::const_vec::ConstVec;
use crate::util::unit::KIBIBYTE;

use super::bank::bank_number::ChrBankRegisterId;
use super::window::ChrWindow;

#[derive(Clone)]
pub struct Layout {
    prg_rom_max_size: u32,
    prg_layout_index: u8,
    prg_layouts: ConstVec<PrgLayout, 16>,
    prg_rom_outer_bank_layout: OuterBankLayout,
    prg_rom_bank_size_override: Option<PrgWindowSize>,

    chr_rom_max_size: u32,
    align_large_chr_windows: bool,
    chr_layout_index: u8,
    chr_layouts: ConstVec<ChrLayout, 16>,
    chr_save_ram_size: u32,
    chr_rom_outer_bank_layout: OuterBankLayout,

    cartridge_selection_name_table_mirrorings: [Option<NameTableMirroring>; 4],
    name_table_mirrorings: &'static [NameTableMirroring],
    four_screen_mirroring_definition: Option<NameTableMirroring>,
    fixed_name_table_mirroring: bool,

    bank_register_overrides: ConstVec<(PrgBankRegisterId, BankNumber), 5>,
    chr_bank_register_overrides: ConstVec<(ChrBankRegisterId, BankNumber), 5>,
    chr_meta_register_overrides: ConstVec<(MetaRegisterId, ChrBankRegisterId), 5>,

    default_chr_source: ChrSource,
}

impl Layout {
    pub const fn builder() -> LayoutBuilder {
        LayoutBuilder::new()
    }

    pub fn make_mapper_params(self, metadata: &ResolvedMetadata, cartridge: &Cartridge, allow_saving: bool)
            -> Result<(PrgMemory, ChrMemory, &'static [NameTableMirroring]), String> {
        let prg_rom_size = cartridge.prg_rom().size();
        if prg_rom_size > self.prg_rom_max_size {
            return Err(format!("PRG ROM size of {}KiB is too large for this mapper.", prg_rom_size / KIBIBYTE));
        }

        let chr_rom_size = cartridge.chr_rom().size();
        if chr_rom_size > self.chr_rom_max_size {
            return Err(format!("CHR ROM size of {}KiB is too large for this mapper.", chr_rom_size / KIBIBYTE));
        }

        let chr_ram = if self.chr_save_ram_size > 0 {
            match metadata.chr_work_ram_size + metadata.chr_save_ram_size {
                0 => RawMemory::new(self.chr_save_ram_size),
                size if size == self.chr_save_ram_size => RawMemory::new(self.chr_save_ram_size),
                _ => panic!("CHR SAVE RAM size from cartridge did not match mapper override value."),
            }
        } else {
            RawMemory::new(metadata.chr_work_ram_size + metadata.chr_save_ram_size)
        };

        let prg_save_ram_page_count = if metadata.prg_save_ram_size > 0 && metadata.prg_save_ram_size < 8 * KIBIBYTE {
            1
        } else {
            u16::try_from(metadata.prg_save_ram_size / (8 * KIBIBYTE)).unwrap()
        };


        let cartridge_has_ram = metadata.prg_work_ram_size > 0 ||  metadata.prg_save_ram_size > 0;
        let mut prg_bank_registers = PrgBankRegisters::new(cartridge_has_ram, prg_save_ram_page_count);
        for (register_id, bank_number) in self.bank_register_overrides.as_iter() {
            prg_bank_registers.set(register_id, bank_number);
        }

        let mut chr_bank_registers = ChrBankRegisters::new(chr_rom_size > 0, !chr_ram.is_empty(), self.default_chr_source);
        for (register_id, bank_number) in self.chr_bank_register_overrides.as_iter() {
            chr_bank_registers.set(register_id, bank_number);
        }

        for (meta_id, register_id) in self.chr_meta_register_overrides.as_iter() {
            chr_bank_registers.set_meta_chr(meta_id, register_id);
        }

        let prg_memory = PrgMemory::new(
            self.prg_layouts.as_iter().collect(),
            self.prg_layout_index,
            cartridge.prg_rom().clone(),
            self.prg_rom_outer_bank_layout,
            self.prg_rom_bank_size_override,
            RawMemory::new(metadata.prg_work_ram_size),
            SaveRam::open(&cartridge.path().to_prg_save_ram_file_path(), metadata.prg_save_ram_size, allow_saving),
            prg_bank_registers,
        );

        let name_table_mirroring = metadata.name_table_mirroring
            .expect("Four screen mirroring specified, but mapper didn't provide a definition of four screen mirroring.");

        let chr_memory = ChrMemory::new(
            self.chr_layouts.as_iter().collect(),
            self.chr_layout_index,
            self.align_large_chr_windows,
            self.chr_rom_outer_bank_layout.outer_bank_count(chr_rom_size),
            cartridge.chr_rom().clone(),
            chr_ram,
            name_table_mirroring,
            self.fixed_name_table_mirroring,
            chr_bank_registers,
        );

        Ok((prg_memory, chr_memory, self.name_table_mirrorings))
    }

    pub fn cartridge_selection_name_table_mirrorings(&self) -> [Option<NameTableMirroring>; 4] {
        self.cartridge_selection_name_table_mirrorings
    }

    pub fn four_screen_mirroring_definition(&self) -> Option<NameTableMirroring> {
        self.four_screen_mirroring_definition
    }

    pub fn supports_prg_ram(&self) -> bool {
        self.prg_layouts.as_iter().any(|prg_layout| prg_layout.supports_ram())
    }
}

#[derive(Clone, Copy)]
pub struct LayoutBuilder {
    prg_rom_max_size: Option<u32>,
    prg_layouts: ConstVec<PrgLayout, 16>,
    prg_layout_index: u8,
    prg_rom_outer_bank_layout: Option<OuterBankLayout>,
    prg_rom_bank_size_override: Option<PrgWindowSize>,

    chr_rom_max_size: Option<u32>,
    chr_layouts: ConstVec<ChrLayout, 16>,
    chr_layout_index: u8,
    chr_rom_outer_bank_layout:  Option<OuterBankLayout>,
    align_large_chr_windows: bool,
    chr_save_ram_size: u32,

    cartridge_selection_name_table_mirrorings: [Option<NameTableMirroring>; 4],
    name_table_mirrorings: &'static [NameTableMirroring],
    four_screen_mirroring_definition: Option<NameTableMirroring>,
    fixed_name_table_mirroring: Option<bool>,

    bank_register_overrides: ConstVec<(PrgBankRegisterId, BankNumber), 5>,
    chr_bank_register_overrides: ConstVec<(ChrBankRegisterId, BankNumber), 5>,
    chr_meta_register_overrides: ConstVec<(MetaRegisterId, ChrBankRegisterId), 5>,
    default_chr_source: ChrSource,
}

impl LayoutBuilder {
    const fn new() -> LayoutBuilder {
        LayoutBuilder {
            prg_rom_max_size: None,
            prg_layout_index: 0,
            prg_layouts: ConstVec::new(),
            prg_rom_outer_bank_layout: None,
            prg_rom_bank_size_override: None,

            chr_rom_max_size: None,
            align_large_chr_windows: true,
            chr_layout_index: 0,
            chr_layouts: ConstVec::new(),
            chr_rom_outer_bank_layout: None,
            chr_save_ram_size: 0,

            // The vast majority of mappers associate these values with the corresponding iNES mirroring bits.
            cartridge_selection_name_table_mirrorings: [
                Some(NameTableMirroring::HORIZONTAL),
                Some(NameTableMirroring::VERTICAL),
                // Four screen
                None,
                // Four screen
                None,
            ],
            name_table_mirrorings: &[],
            four_screen_mirroring_definition: None,
            fixed_name_table_mirroring: None,

            bank_register_overrides: ConstVec::new(),
            chr_bank_register_overrides: ConstVec::new(),
            chr_meta_register_overrides: ConstVec::new(),

            default_chr_source: ChrSource::RomOrRam,
        }
    }

    pub const fn prg_rom_max_size(&mut self, value: u32) -> &mut Self {
        self.prg_rom_max_size = Some(value);
        self
    }

    pub const fn prg_layout_index(&mut self, value: u8) -> &mut Self  {
        self.prg_layout_index = value;
        self
    }

    pub const fn prg_layout(&mut self, windows: &'static [PrgWindow]) -> &mut Self {
        self.prg_layouts.push(PrgLayout::new(windows));
        self
    }

    pub const fn prg_outer_bank_count(&mut self, count: u8) -> &mut Self {
        self.prg_rom_outer_bank_layout = Some(OuterBankLayout::ExactCount(NonZeroU8::new(count).unwrap()));
        self
    }

    pub const fn prg_rom_outer_bank_size(&mut self, size: u32) -> &mut Self {
        self.prg_rom_outer_bank_layout = Some(OuterBankLayout::Size(size));
        self
    }

    pub const fn prg_rom_bank_size_override(&mut self, size: u32) -> &mut Self {
        self.prg_rom_bank_size_override = Some(PrgWindowSize::from_raw(size));
        self
    }

    pub const fn chr_rom_max_size(&mut self, value: u32) -> &mut Self {
        self.chr_rom_max_size = Some(value);
        self
    }

    pub const fn chr_layout(&mut self, windows: &'static [ChrWindow]) -> &mut Self {
        self.chr_layouts.push(ChrLayout::new(windows));
        self
    }

    pub const fn chr_layout_index(&mut self, value: u8) -> &mut Self {
        self.chr_layout_index = value;
        self
    }

    pub const fn do_not_align_large_chr_windows(&mut self) -> &mut Self {
        self.align_large_chr_windows = false;
        self
    }

    pub const fn chr_save_ram_size(&mut self, value: u32) -> &mut Self {
        self.chr_save_ram_size = value;
        self
    }

    pub const fn chr_rom_outer_bank_size(&mut self, size: u32) -> &mut Self {
        self.chr_rom_outer_bank_layout = Some(OuterBankLayout::Size(size));
        self
    }

    pub const fn cartridge_selection_name_table_mirrorings(&mut self, value: [Option<NameTableMirroring>; 4]) -> &mut Self {
        self.cartridge_selection_name_table_mirrorings = value;
        self
    }

    pub const fn name_table_mirrorings(&mut self, value: &'static [NameTableMirroring]) -> &mut Self {
        self.name_table_mirrorings = value;
        self
    }

    pub const fn four_screen_mirroring_definition(&mut self, value: NameTableMirroring) -> &mut Self {
        self.four_screen_mirroring_definition = Some(value);
        self
    }

    pub const fn fixed_name_table_mirroring(&mut self) -> &mut Self {
        self.fixed_name_table_mirroring = Some(true);
        self
    }

    pub const fn complicated_name_table_mirroring(&mut self) -> &mut Self {
        self.fixed_name_table_mirroring = Some(false);
        self
    }

    pub const fn override_prg_bank_register(
        &mut self,
        id: PrgBankRegisterId,
        bank_number: i16,
    ) -> &mut LayoutBuilder {
        self.bank_register_overrides.push((id, BankNumber::from_i16(bank_number)));
        self
    }

    pub const fn override_chr_bank_register(
        &mut self,
        id: ChrBankRegisterId,
        bank_number: i16,
    ) -> &mut LayoutBuilder {
        self.chr_bank_register_overrides.push((id, BankNumber::from_i16(bank_number)));
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

    pub const fn default_chr_source(&mut self, chr_source: ChrSource) -> &mut LayoutBuilder {
        self.default_chr_source = chr_source;
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

        let fixed_name_table_mirroring = match (self.name_table_mirrorings, self.fixed_name_table_mirroring) {
            ([_,..], None       ) => false,
            ([]    , Some(fixed)) => fixed,
            _ => panic!("Must set one of name_table_mirrorings, fixed_name_table_mirroring, or complicated_name_table_mirroring"),
        };

        Layout {
            prg_rom_max_size: self.prg_rom_max_size.expect("prg_rom_max_size must be set"),
            prg_layouts: self.prg_layouts,
            prg_layout_index: self.prg_layout_index,
            prg_rom_outer_bank_layout,
            prg_rom_bank_size_override: self.prg_rom_bank_size_override,

            chr_rom_max_size: self.chr_rom_max_size.expect("chr_rom_max_size must be set"),
            chr_layouts: self.chr_layouts,
            chr_layout_index: self.chr_layout_index,
            align_large_chr_windows: self.align_large_chr_windows,
            chr_save_ram_size: self.chr_save_ram_size,
            chr_rom_outer_bank_layout,

            cartridge_selection_name_table_mirrorings: self.cartridge_selection_name_table_mirrorings,
            name_table_mirrorings: self.name_table_mirrorings,
            four_screen_mirroring_definition: self.four_screen_mirroring_definition,
            fixed_name_table_mirroring,

            bank_register_overrides: self.bank_register_overrides,
            chr_bank_register_overrides: self.chr_bank_register_overrides,
            chr_meta_register_overrides: self.chr_meta_register_overrides,

            default_chr_source: self.default_chr_source,
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum OuterBankLayout {
    ExactCount(NonZeroU8),
    Size(u32),
}

impl OuterBankLayout {
    const SINGLE_BANK: Self = Self::ExactCount(NonZeroU8::new(1).unwrap());

    pub fn outer_bank_count(self, memory_size: u32) -> NonZeroU8 {
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