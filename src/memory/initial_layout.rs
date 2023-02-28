use crate::cartridge::Cartridge;
use crate::memory::board::Board;
use crate::memory::cpu::prg_memory::{PrgLayout, PrgMemory, PrgWindow};
use crate::memory::mapper::MapperParams;
use crate::memory::ppu::chr_memory::{ChrLayout, ChrMemory, ChrWindow};
use crate::ppu::name_table::name_table_mirroring::NameTableMirroring;

pub struct InitialLayout {
    pub prg_max_bank_count: u16,
    pub prg_bank_size: usize,
    pub prg_windows_by_board: &'static[(Board, &'static [PrgWindow])],

    pub chr_max_bank_count: u16,
    pub chr_bank_size: usize,
    pub chr_windows: &'static [ChrWindow],

    pub name_table_mirroring_source: NameTableMirroringSource,
}

impl InitialLayout {
    pub fn make_mapper_params(&self, cartridge: &Cartridge, board: Board) -> MapperParams {
        let prg_windows = self.lookup_prg_windows_by_board(board);
        let prg_layout = PrgLayout::new(self.prg_max_bank_count, self.prg_bank_size, prg_windows.to_vec());
        let prg_memory = PrgMemory::new(prg_layout, cartridge.prg_rom());

        let chr_layout = ChrLayout::new(self.chr_max_bank_count, self.chr_bank_size, self.chr_windows.to_vec());
        let chr_memory = ChrMemory::new(chr_layout, cartridge.chr_rom());

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

pub enum NameTableMirroringSource {
    Direct(NameTableMirroring),
    Cartridge,
}
