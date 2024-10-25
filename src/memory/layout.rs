use crate::cartridge::cartridge::Cartridge;
use crate::memory::cpu::prg_memory::{PrgMemory, PrgLayout};
use crate::memory::mapper::MapperParams;
use crate::memory::ppu::chr_memory::{ChrMemory, ChrLayout};
use crate::ppu::name_table::name_table_mirroring::NameTableMirroring;
use crate::memory::bank::bank_index::{BankIndex, BankRegisters, MetaRegisterId, BankRegisterId};

pub struct Layout {
    prg_max_bank_count: u16,
    prg_bank_size: usize,
    prg_layout: PrgLayout,

    chr_max_bank_count: u16,
    chr_bank_size: usize,
    chr_layout: ChrLayout,
    align_large_chr_layout: bool,

    name_table_mirroring_source: NameTableMirroringSource,
    bank_register_override: Option<(BankRegisterId, BankIndex)>,
    meta_register_override: (MetaRegisterId, BankRegisterId),
    second_meta_register_override: (MetaRegisterId, BankRegisterId),
}

impl Layout {
    pub const fn builder() -> LayoutBuilder {
        LayoutBuilder::new()
    }

    pub fn make_mapper_params(&self, cartridge: &Cartridge) -> MapperParams {
        let prg_memory = PrgMemory::new(
            self.prg_layout,
            self.prg_bank_size,
            cartridge.prg_rom().to_vec(),
        );
        let chr_memory = ChrMemory::new(
            self.chr_layout,
            self.chr_bank_size,
            self.align_large_chr_layout,
            cartridge.chr_rom().to_vec(),
        );

        let name_table_mirroring = match self.name_table_mirroring_source {
            NameTableMirroringSource::Direct(mirroring) => mirroring,
            NameTableMirroringSource::Cartridge => cartridge.name_table_mirroring(),
        };

        let mut bank_registers = BankRegisters::new();
        if let Some((register_id, bank_index)) = self.bank_register_override {
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
    prg_max_bank_count: Option<u16>,
    prg_bank_size: Option<usize>,
    prg_layout: Option<PrgLayout>,

    chr_max_bank_count: Option<u16>,
    chr_bank_size: Option<usize>,
    chr_layout: Option<ChrLayout>,
    align_large_chr_layout: bool,

    name_table_mirroring_source: Option<NameTableMirroringSource>,
    bank_register_override: Option<(BankRegisterId, BankIndex)>,
    meta_register_override: (MetaRegisterId, BankRegisterId),
    // Can't clone a map in a const context, so each override must be a separate field.
    second_meta_register_override: (MetaRegisterId, BankRegisterId),
}

impl LayoutBuilder {
    const fn new() -> LayoutBuilder {
        LayoutBuilder {
            prg_max_bank_count: None,
            prg_bank_size: None,
            prg_layout: None,

            chr_max_bank_count: None,
            chr_bank_size: None,
            chr_layout: None,
            align_large_chr_layout: true,

            name_table_mirroring_source: None,
            bank_register_override: None,
            meta_register_override: (MetaRegisterId::M0, BankRegisterId::C0),
            second_meta_register_override: (MetaRegisterId::M1, BankRegisterId::C0),
        }
    }

    pub const fn prg_max_bank_count(&mut self, value: u16) -> &mut LayoutBuilder {
        self.prg_max_bank_count = Some(value);
        self
    }

    pub const fn prg_bank_size(&mut self, value: usize) -> &mut LayoutBuilder {
        self.prg_bank_size = Some(value);
        self
    }

    pub const fn prg_layout(&mut self, value: PrgLayout) -> &mut LayoutBuilder {
        self.prg_layout = Some(value);
        self
    }

    pub const fn chr_max_bank_count(&mut self, value: u16) -> &mut LayoutBuilder {
        self.chr_max_bank_count = Some(value);
        self
    }

    pub const fn chr_bank_size(&mut self, value: usize) -> &mut LayoutBuilder {
        self.chr_bank_size = Some(value);
        self
    }

    pub const fn chr_layout(&mut self, value: ChrLayout) -> &mut LayoutBuilder {
        self.chr_layout = Some(value);
        self
    }

    pub const fn do_not_align_large_chr_layout(&mut self) -> &mut LayoutBuilder {
        self.align_large_chr_layout = false;
        self
    }

    pub const fn name_table_mirroring_source(
        &mut self,
        value: NameTableMirroringSource,
    ) -> &mut LayoutBuilder {
        self.name_table_mirroring_source = Some(value);
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
        Layout {
            prg_max_bank_count: self.prg_max_bank_count.unwrap(),
            prg_bank_size: self.prg_bank_size.unwrap(),
            prg_layout: self.prg_layout.unwrap(),

            chr_max_bank_count: self.chr_max_bank_count.unwrap(),
            chr_bank_size: self.chr_bank_size.unwrap(),
            chr_layout: self.chr_layout.unwrap(),
            align_large_chr_layout: self.align_large_chr_layout,

            name_table_mirroring_source: self.name_table_mirroring_source.unwrap(),
            bank_register_override: self.bank_register_override,
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
