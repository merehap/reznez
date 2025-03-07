use std::collections::BTreeSet;

use crate::cartridge::cartridge::Cartridge;
use crate::memory::bank::bank_index::{BankIndex, BankRegisters, MetaRegisterId, BankRegisterId};
use crate::memory::cpu::prg_layout::PrgLayout;
use crate::memory::cpu::prg_memory::PrgMemory;
use crate::memory::irq_source::IrqSource;
use crate::memory::mapper::{MapperParams, RamStatus};
use crate::memory::ppu::chr_layout::ChrLayout;
use crate::memory::ppu::chr_memory::ChrMemory;
use crate::memory::window::{RamStatusInfo, Window};
use crate::ppu::name_table::name_table_mirroring::NameTableMirroring;
use crate::util::const_vec::ConstVec;
use crate::util::unit::KIBIBYTE;

#[derive(Clone)]
pub struct Layout {
    prg_max_size: u32,
    prg_bank_size_override: Option<u16>,
    prg_layout_index: u8,
    prg_layouts: ConstVec<PrgLayout, 10>,

    chr_max_size: u32,
    align_large_chr_windows: bool,
    chr_layout_index: u8,
    chr_layouts: ConstVec<ChrLayout, 10>,

    name_table_mirroring_source: NameTableMirroringSource,
    name_table_mirrorings: &'static [NameTableMirroring],

    ram_statuses: &'static [RamStatus],

    bank_register_overrides: ConstVec<(BankRegisterId, BankIndex), 5>,
    meta_register_overrides: ConstVec<(MetaRegisterId, BankRegisterId), 5>,
}

impl Layout {
    pub const fn builder() -> LayoutBuilder {
        LayoutBuilder::new()
    }

    pub fn make_mapper_params(self, cartridge: &Cartridge) -> MapperParams {
        let prg_size = cartridge.prg_rom().size();
        assert!(prg_size <= self.prg_max_size, "PRG size of {}KiB is too large for this mapper.", prg_size / KIBIBYTE);
        let chr_size = cartridge.chr_rom().size();
        assert!(chr_size <= self.chr_max_size, "CHR size of {}KiB is too large for this mapper.", chr_size / KIBIBYTE);

        let prg_memory = PrgMemory::new(
            self.prg_layouts.as_iter().collect(),
            self.prg_layout_index,
            self.prg_bank_size_override,
            cartridge.prg_rom().clone(),
            // TODO: Work RAM and Save RAM should be separate, but are combined here.
            cartridge.prg_ram_size() + cartridge.prg_nvram_size(),
        );
        let chr_memory = ChrMemory::new(
            self.chr_layouts.as_iter().collect(),
            self.chr_layout_index,
            self.align_large_chr_windows,
            cartridge.chr_rom().clone(),
        );

        let name_table_mirroring = match self.name_table_mirroring_source {
            NameTableMirroringSource::Direct(mirroring) => mirroring,
            NameTableMirroringSource::Cartridge => cartridge.name_table_mirroring(),
        };

        let mut bank_registers = BankRegisters::new();
        for (register_id, bank_index) in self.bank_register_overrides.as_iter() {
            bank_registers.set(register_id, bank_index);
        }

        for (meta_id, register_id) in self.meta_register_overrides.as_iter() {
            bank_registers.set_meta(meta_id, register_id);
        }

        let mut ram_not_present = BTreeSet::new();
        if cartridge.prg_ram_size() == 0 && cartridge.prg_nvram_size() == 0 {
            for status_info in prg_memory.ram_status_infos() {
                match status_info {
                    RamStatusInfo::Absent | RamStatusInfo::MapperCustom { .. } => { /* Do nothing. */ }
                    RamStatusInfo::PossiblyPresent { register_id, status_on_absent } => {
                        bank_registers.set_ram_status(register_id, status_on_absent);
                        ram_not_present.insert(register_id);
                    }
                }
            }
        }

        if cartridge.chr_ram_size() == 0 && cartridge.chr_nvram_size() == 0 {
            for status_info in chr_memory.ram_status_infos() {
                match status_info {
                    RamStatusInfo::Absent | RamStatusInfo::MapperCustom { .. } => { /* Do nothing. */ }
                    RamStatusInfo::PossiblyPresent { register_id, status_on_absent } => {
                        bank_registers.set_ram_status(register_id, status_on_absent);
                        ram_not_present.insert(register_id);
                    }
                }
            }
        }

        MapperParams {
            prg_memory,
            chr_memory,
            bank_registers,
            name_table_mirroring,
            name_table_mirrorings: self.name_table_mirrorings,
            ram_statuses: self.ram_statuses,
            ram_not_present,
            irq: IrqSource::new(),
        }
    }
}

#[derive(Clone, Copy)]
pub struct LayoutBuilder {
    prg_max_size: Option<u32>,
    prg_bank_size_override: Option<u16>,
    prg_layouts: ConstVec<PrgLayout, 10>,
    prg_layout_index: u8,

    chr_max_size: Option<u32>,
    chr_layouts: ConstVec<ChrLayout, 10>,
    chr_layout_index: u8,
    align_large_chr_windows: bool,

    name_table_mirroring_source: NameTableMirroringSource,
    name_table_mirrorings: &'static [NameTableMirroring],

    ram_statuses: &'static [RamStatus],

    bank_register_overrides: ConstVec<(BankRegisterId, BankIndex), 5>,
    meta_register_overrides: ConstVec<(MetaRegisterId, BankRegisterId), 5>,
}

impl LayoutBuilder {
    const fn new() -> LayoutBuilder {
        LayoutBuilder {
            prg_max_size: None,
            prg_bank_size_override: None,
            prg_layout_index: 0,
            prg_layouts: ConstVec::new(),

            chr_max_size: None,
            align_large_chr_windows: true,
            chr_layout_index: 0,
            chr_layouts: ConstVec::new(),

            name_table_mirroring_source: NameTableMirroringSource::Cartridge,
            name_table_mirrorings: &[],

            ram_statuses: &[],

            bank_register_overrides: ConstVec::new(),
            meta_register_overrides: ConstVec::new(),
        }
    }

    pub const fn prg_max_size(&mut self, value: u32) -> &mut LayoutBuilder {
        self.prg_max_size = Some(value);
        self
    }

    pub const fn prg_bank_size_override(&mut self, value: u32) -> &mut LayoutBuilder {
        self.prg_bank_size_override = Some(value as u16);
        self
    }

    pub const fn prg_layout_index(&mut self, value: u8) -> &mut LayoutBuilder {
        self.prg_layout_index = value;
        self
    }

    pub const fn prg_layout(&mut self, windows: &'static [Window]) -> &mut LayoutBuilder {
        self.prg_layouts.push(PrgLayout::new(windows));
        self
    }

    pub const fn chr_max_size(&mut self, value: u32) -> &mut LayoutBuilder {
        self.chr_max_size = Some(value);
        self
    }

    pub const fn chr_layout(&mut self, windows: &'static [Window]) -> &mut LayoutBuilder {
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

    pub const fn override_initial_name_table_mirroring(
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

    pub const fn ram_statuses(
        &mut self,
        value: &'static [RamStatus],
    ) -> &mut LayoutBuilder {
        self.ram_statuses = value;
        self
    }

    pub const fn override_bank_register(
        &mut self,
        id: BankRegisterId,
        bank_index: i16,
    ) -> &mut LayoutBuilder {
        self.bank_register_overrides.push((id, BankIndex::from_i16(bank_index)));
        self
    }

    pub const fn override_meta_register(
        &mut self,
        meta_id: MetaRegisterId,
        id: BankRegisterId,
    ) -> &mut LayoutBuilder {
        self.meta_register_overrides.push((meta_id, id));
        self
    }

    pub const fn build(self) -> Layout {
        assert!(!self.prg_layouts.is_empty());
        assert!(!self.chr_layouts.is_empty());

        Layout {
            prg_max_size: self.prg_max_size.unwrap(),
            prg_bank_size_override: self.prg_bank_size_override,
            prg_layouts: self.prg_layouts,
            prg_layout_index: self.prg_layout_index,

            chr_max_size: self.chr_max_size.unwrap(),
            chr_layouts: self.chr_layouts,
            chr_layout_index: self.chr_layout_index,
            align_large_chr_windows: self.align_large_chr_windows,

            name_table_mirroring_source: self.name_table_mirroring_source,
            name_table_mirrorings: self.name_table_mirrorings,

            ram_statuses: self.ram_statuses,

            bank_register_overrides: self.bank_register_overrides,
            meta_register_overrides: self.meta_register_overrides,
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
