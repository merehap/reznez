use crate::cartridge::Cartridge;
use crate::memory::board::Board;
use crate::memory::cpu::prg_memory::{PrgMemory, PrgWindow};
use crate::memory::mapper::MapperParams;
use crate::memory::ppu::chr_memory::{ChrLayout, ChrMemory, ChrWindow};
use crate::ppu::name_table::name_table_mirroring::NameTableMirroring;

pub struct InitialLayout {
    prg_max_bank_count: u16,
    prg_bank_size: usize,
    prg_windows_by_board: &'static[(Board, &'static [PrgWindow])],

    chr_max_bank_count: u16,
    chr_bank_size: usize,
    chr_windows: &'static [ChrWindow],
    align_large_chr_windows: bool,

    name_table_mirroring_source: NameTableMirroringSource,
}

impl InitialLayout {
    pub const fn builder() -> InitialLayoutBuilder {
        InitialLayoutBuilder::new()
    }

    pub fn make_mapper_params(&'static self, cartridge: &Cartridge, board: Board) -> MapperParams {
        let prg_windows = self.lookup_prg_windows_by_board(board);
        let prg_memory = PrgMemory::new(
            prg_windows,
            self.prg_max_bank_count,
            self.prg_bank_size,
            cartridge.prg_rom(),
        );

        let chr_layout = ChrLayout::new(self.chr_max_bank_count, self.chr_bank_size, self.chr_windows.to_vec());
        let chr_memory = ChrMemory::new(chr_layout, self.align_large_chr_windows, cartridge.chr_rom());

        let name_table_mirroring = match self.name_table_mirroring_source {
            NameTableMirroringSource::Direct(mirroring) => mirroring,
            NameTableMirroringSource::Cartridge => cartridge.name_table_mirroring(),
        };

        MapperParams { prg_memory, chr_memory, name_table_mirroring }
    }

    fn lookup_prg_windows_by_board(&self, target: Board) -> &[PrgWindow] {
        for &(board, prg_windows) in self.prg_windows_by_board {
            if board == target {
                return prg_windows;
            }
        }

        panic!("Board {target:?} is not configured for this mapper.");
    }
}

#[derive(Clone, Copy)]
pub struct InitialLayoutBuilder {
    prg_max_bank_count: Option<u16>,
    prg_bank_size: Option<usize>,
    prg_windows_by_board: Option<&'static[(Board, &'static [PrgWindow])]>,

    chr_max_bank_count: Option<u16>,
    chr_bank_size: Option<usize>,
    chr_windows: Option<&'static [ChrWindow]>,
    align_large_chr_windows: bool,

    name_table_mirroring_source: Option<NameTableMirroringSource>,
}

impl InitialLayoutBuilder {
    const fn new() -> InitialLayoutBuilder {
        InitialLayoutBuilder {
            prg_max_bank_count: None,
            prg_bank_size: None,
            prg_windows_by_board: None,

            chr_max_bank_count: None,
            chr_bank_size: None,
            chr_windows: None,
            align_large_chr_windows: true,

            name_table_mirroring_source: None,
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

    pub const fn prg_windows_by_board(
        &mut self,
        value: &'static[(Board, &'static [PrgWindow])],
    ) -> &mut InitialLayoutBuilder {
        self.prg_windows_by_board = Some(value);
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

    pub const fn chr_windows(&mut self, value: &'static [ChrWindow]) -> &mut InitialLayoutBuilder {
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

    pub const fn build(self) -> InitialLayout {
        InitialLayout {
            prg_max_bank_count: self.prg_max_bank_count.unwrap(),
            prg_bank_size: self.prg_bank_size.unwrap(),
            prg_windows_by_board: self.prg_windows_by_board.unwrap(),

            chr_max_bank_count: self.chr_max_bank_count.unwrap(),
            chr_bank_size: self.chr_bank_size.unwrap(),
            chr_windows: self.chr_windows.unwrap(),
            align_large_chr_windows: self.align_large_chr_windows,

            name_table_mirroring_source: self.name_table_mirroring_source.unwrap(),
        }
    }
}

#[derive(Clone, Copy)]
pub enum NameTableMirroringSource {
    Direct(NameTableMirroring),
    Cartridge,
}
