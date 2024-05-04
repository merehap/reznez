use crate::cartridge::cartridge::Cartridge;
use crate::memory::cpu::prg_memory::{PrgMemory, PrgLayout};
use crate::memory::mapper::MapperParams;
use crate::memory::ppu::chr_memory::{ChrMemory, ChrLayout};
use crate::ppu::name_table::name_table_mirroring::NameTableMirroring;
use crate::memory::bank_index::{BankIndex, BankIndexRegisters, MetaRegisterId, BankIndexRegisterId};

pub struct InitialLayout {
    prg_max_bank_count: u16,
    prg_bank_size: usize,
    prg_windows: PrgLayout,

    chr_max_bank_count: u16,
    chr_bank_size: usize,
    chr_windows: ChrLayout,
    align_large_chr_windows: bool,

    name_table_mirroring_source: NameTableMirroringSource,
    bank_index_register_override: Option<(BankIndexRegisterId, BankIndex)>,
    meta_register_override: (MetaRegisterId, BankIndexRegisterId),
    second_meta_register_override: (MetaRegisterId, BankIndexRegisterId),
}

impl InitialLayout {
    pub const fn builder() -> InitialLayoutBuilder {
        InitialLayoutBuilder::new()
    }

    pub fn make_mapper_params(&self, cartridge: &Cartridge) -> MapperParams {
        let prg_memory = PrgMemory::new(
            self.prg_windows,
            self.prg_max_bank_count,
            self.prg_bank_size,
            cartridge.prg_rom().to_vec(),
        );
        let chr_memory = ChrMemory::new(
            self.chr_windows,
            self.chr_max_bank_count,
            self.chr_bank_size,
            self.align_large_chr_windows,
            cartridge.chr_rom().to_vec(),
        );

        let name_table_mirroring = match self.name_table_mirroring_source {
            NameTableMirroringSource::Direct(mirroring) => mirroring,
            NameTableMirroringSource::Cartridge => cartridge.name_table_mirroring(),
        };

        let mut bank_index_registers = BankIndexRegisters::new();
        if let Some((register_id, bank_index)) = self.bank_index_register_override {
            bank_index_registers.set(register_id, bank_index);
        }

        bank_index_registers.set_meta(self.meta_register_override.0, self.meta_register_override.1);
        bank_index_registers.set_meta(self.second_meta_register_override.0, self.second_meta_register_override.1);

        MapperParams {
            prg_memory,
            chr_memory,
            bank_index_registers,
            name_table_mirroring,
        }
    }
}

#[derive(Clone, Copy)]
pub struct InitialLayoutBuilder {
    prg_max_bank_count: Option<u16>,
    prg_bank_size: Option<usize>,
    prg_windows: Option<PrgLayout>,

    chr_max_bank_count: Option<u16>,
    chr_bank_size: Option<usize>,
    chr_windows: Option<ChrLayout>,
    align_large_chr_windows: bool,

    name_table_mirroring_source: Option<NameTableMirroringSource>,
    bank_index_register_override: Option<(BankIndexRegisterId, BankIndex)>,
    meta_register_override: (MetaRegisterId, BankIndexRegisterId),
    // Can't clone a map in a const context, so each override must be a separate field.
    second_meta_register_override: (MetaRegisterId, BankIndexRegisterId),
}

impl InitialLayoutBuilder {
    const fn new() -> InitialLayoutBuilder {
        InitialLayoutBuilder {
            prg_max_bank_count: None,
            prg_bank_size: None,
            prg_windows: None,

            chr_max_bank_count: None,
            chr_bank_size: None,
            chr_windows: None,
            align_large_chr_windows: true,

            name_table_mirroring_source: None,
            bank_index_register_override: None,
            meta_register_override: (MetaRegisterId::M0, BankIndexRegisterId::C0),
            second_meta_register_override: (MetaRegisterId::M1, BankIndexRegisterId::C0),
        }
    }

    pub const fn prg_max_bank_count(&mut self, value: u16) -> &mut InitialLayoutBuilder {
        self.prg_max_bank_count = Some(value);
        self
    }

    pub const fn prg_bank_size(&mut self, value: usize) -> &mut InitialLayoutBuilder {
        self.prg_bank_size = Some(value);
        self
    }

    pub const fn prg_windows(&mut self, value: PrgLayout) -> &mut InitialLayoutBuilder {
        self.prg_windows = Some(value);
        self
    }

    pub const fn chr_max_bank_count(&mut self, value: u16) -> &mut InitialLayoutBuilder {
        self.chr_max_bank_count = Some(value);
        self
    }

    pub const fn chr_bank_size(&mut self, value: usize) -> &mut InitialLayoutBuilder {
        self.chr_bank_size = Some(value);
        self
    }

    pub const fn chr_windows(&mut self, value: ChrLayout) -> &mut InitialLayoutBuilder {
        self.chr_windows = Some(value);
        self
    }

    pub const fn do_not_align_large_chr_windows(&mut self) -> &mut InitialLayoutBuilder {
        self.align_large_chr_windows = false;
        self
    }

    pub const fn name_table_mirroring_source(
        &mut self,
        value: NameTableMirroringSource,
    ) -> &mut InitialLayoutBuilder {
        self.name_table_mirroring_source = Some(value);
        self
    }

    pub const fn override_bank_index_register(
        &mut self,
        id: BankIndexRegisterId,
        bank_index: BankIndex,
    ) -> &mut InitialLayoutBuilder {
        self.bank_index_register_override = Some((id, bank_index));
        self
    }

    pub const fn override_meta_register(
        &mut self,
        meta_id: MetaRegisterId,
        id: BankIndexRegisterId,
    ) -> &mut InitialLayoutBuilder {
        self.meta_register_override = (meta_id, id);
        self
    }

    pub const fn override_second_meta_register(
        &mut self,
        meta_id: MetaRegisterId,
        id: BankIndexRegisterId,
    ) -> &mut InitialLayoutBuilder {
        self.second_meta_register_override = (meta_id, id);
        self
    }

    pub const fn build(self) -> InitialLayout {
        InitialLayout {
            prg_max_bank_count: self.prg_max_bank_count.unwrap(),
            prg_bank_size: self.prg_bank_size.unwrap(),
            prg_windows: self.prg_windows.unwrap(),

            chr_max_bank_count: self.chr_max_bank_count.unwrap(),
            chr_bank_size: self.chr_bank_size.unwrap(),
            chr_windows: self.chr_windows.unwrap(),
            align_large_chr_windows: self.align_large_chr_windows,

            name_table_mirroring_source: self.name_table_mirroring_source.unwrap(),
            bank_index_register_override: self.bank_index_register_override,
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
