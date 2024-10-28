use crate::cartridge::cartridge::Cartridge;
use crate::memory::cpu::prg_memory::{PrgMemory, PrgLayout};
use crate::memory::mapper::MapperParams;
use crate::memory::ppu::chr_memory::{ChrMemory, ChrLayout};
use crate::ppu::name_table::name_table_mirroring::NameTableMirroring;
use crate::memory::bank::bank_index::{BankIndex, BankRegisters, MetaRegisterId, BankRegisterId};
use crate::util::const_vec::ConstVec;

pub struct Layout {
    prg_max_size: u32,
    prg_layouts: ConstVec<PrgLayout, 10>,
    prg_layout_index: usize,

    chr_max_size: u32,
    chr_layouts: ConstVec<ChrLayout, 10>,
    chr_layout_index: usize,
    align_large_chr_layout: bool,

    name_table_mirroring_source: NameTableMirroringSource,

    // TODO: Replace these with ConstVecs.
    bank_register_override: Option<(BankRegisterId, BankIndex)>,
    second_bank_register_override: Option<(BankRegisterId, BankIndex)>,
    meta_register_override: (MetaRegisterId, BankRegisterId),
    second_meta_register_override: (MetaRegisterId, BankRegisterId),
}

impl Layout {
    pub const fn builder() -> LayoutBuilder {
        LayoutBuilder::new()
    }

    pub fn make_mapper_params(self, cartridge: &Cartridge) -> MapperParams {
        let prg_memory = PrgMemory::new(
            self.prg_layouts.into_vec(),
            self.prg_layout_index,
            cartridge.prg_rom().clone(),
        );
        let chr_memory = ChrMemory::new(
            self.chr_layouts.into_vec(),
            self.chr_layout_index,
            self.align_large_chr_layout,
            cartridge.chr_rom().clone(),
        );

        let name_table_mirroring = match self.name_table_mirroring_source {
            NameTableMirroringSource::Direct(mirroring) => mirroring,
            NameTableMirroringSource::Cartridge => cartridge.name_table_mirroring(),
        };

        let mut bank_registers = BankRegisters::new();
        if let Some((register_id, bank_index)) = self.bank_register_override {
            bank_registers.set(register_id, bank_index);
        }

        if let Some((register_id, bank_index)) = self.second_bank_register_override {
            bank_registers.set(register_id, bank_index);
        }

        bank_registers.set_meta(self.meta_register_override.0, self.meta_register_override.1);
        bank_registers.set_meta(self.second_meta_register_override.0, self.second_meta_register_override.1);

        MapperParams {
            prg_memory,
            chr_memory,
            bank_registers,
            name_table_mirroring,
        }
    }
}

#[derive(Clone, Copy)]
pub struct LayoutBuilder {
    prg_max_size: Option<u32>,
    prg_layouts: ConstVec<PrgLayout, 10>,
    prg_layout_index: usize,

    chr_max_size: Option<u32>,
    chr_layouts: ConstVec<ChrLayout, 10>,
    chr_layout_index: usize,
    align_large_chr_layout: bool,

    name_table_mirroring_source: NameTableMirroringSource,
    bank_register_override: Option<(BankRegisterId, BankIndex)>,
    second_bank_register_override: Option<(BankRegisterId, BankIndex)>,
    meta_register_override: (MetaRegisterId, BankRegisterId),
    // Can't clone a map in a const context, so each override must be a separate field.
    second_meta_register_override: (MetaRegisterId, BankRegisterId),
}

impl LayoutBuilder {
    const fn new() -> LayoutBuilder {
        LayoutBuilder {
            prg_max_size: None,
            prg_layouts: ConstVec::new(),
            prg_layout_index: 0,

            chr_max_size: None,
            chr_layouts: ConstVec::new(),
            chr_layout_index: 0,
            align_large_chr_layout: true,

            name_table_mirroring_source: NameTableMirroringSource::Cartridge,
            bank_register_override: None,
            second_bank_register_override: None,
            meta_register_override: (MetaRegisterId::M0, BankRegisterId::C0),
            second_meta_register_override: (MetaRegisterId::M1, BankRegisterId::C0),
        }
    }

    pub const fn prg_max_size(&mut self, value: u32) -> &mut LayoutBuilder {
        self.prg_max_size = Some(value);
        self
    }

    pub const fn prg_layout(&mut self, value: PrgLayout) -> &mut LayoutBuilder {
        self.prg_layouts.push(value);
        self
    }

    pub const fn prg_layout_index(&mut self, value: usize) -> &mut LayoutBuilder {
        self.prg_layout_index = value;
        self
    }

    pub const fn chr_max_size(&mut self, value: u32) -> &mut LayoutBuilder {
        self.chr_max_size = Some(value);
        self
    }

    pub const fn chr_layout(&mut self, value: ChrLayout) -> &mut LayoutBuilder {
        self.chr_layouts.push(value);
        self
    }

    pub const fn chr_layout_index(&mut self, value: usize) -> &mut LayoutBuilder {
        self.chr_layout_index = value;
        self
    }

    pub const fn do_not_align_large_chr_layout(&mut self) -> &mut LayoutBuilder {
        self.align_large_chr_layout = false;
        self
    }

    pub const fn override_initial_name_table_mirroring(
        &mut self,
        value: NameTableMirroring,
    ) -> &mut LayoutBuilder {
        self.name_table_mirroring_source = NameTableMirroringSource::Direct(value);
        self
    }

    pub const fn override_bank_register(
        &mut self,
        id: BankRegisterId,
        bank_index: BankIndex,
    ) -> &mut LayoutBuilder {
        self.bank_register_override = Some((id, bank_index));
        self
    }

    pub const fn override_second_bank_register(
        &mut self,
        id: BankRegisterId,
        bank_index: BankIndex,
    ) -> &mut LayoutBuilder {
        self.second_bank_register_override = Some((id, bank_index));
        self
    }

    pub const fn override_meta_register(
        &mut self,
        meta_id: MetaRegisterId,
        id: BankRegisterId,
    ) -> &mut LayoutBuilder {
        self.meta_register_override = (meta_id, id);
        self
    }

    pub const fn override_second_meta_register(
        &mut self,
        meta_id: MetaRegisterId,
        id: BankRegisterId,
    ) -> &mut LayoutBuilder {
        self.second_meta_register_override = (meta_id, id);
        self
    }

    pub const fn build(self) -> Layout {
        assert!(!self.prg_layouts.is_empty());
        assert!(!self.chr_layouts.is_empty());

        Layout {
            prg_max_size: self.prg_max_size.unwrap(),
            prg_layouts: self.prg_layouts,
            prg_layout_index: self.prg_layout_index,

            chr_max_size: self.chr_max_size.unwrap(),
            chr_layouts: self.chr_layouts,
            chr_layout_index: self.chr_layout_index,
            align_large_chr_layout: self.align_large_chr_layout,

            name_table_mirroring_source: self.name_table_mirroring_source,

            bank_register_override: self.bank_register_override,
            second_bank_register_override: self.second_bank_register_override,
            meta_register_override: self.meta_register_override,
            second_meta_register_override: self.second_meta_register_override,
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
