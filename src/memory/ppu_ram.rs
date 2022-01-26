use crate::memory::palette_ram::PaletteRam;
use crate::memory::vram::Vram;
use crate::ppu::name_table::name_table_mirroring::NameTableMirroring;

pub struct PpuRam {
    pub vram: Vram,
    pub palette_ram: PaletteRam,
    pub name_table_mirroring: NameTableMirroring,
}

impl PpuRam {
    pub fn new(name_table_mirroring: NameTableMirroring) -> PpuRam {
        PpuRam {
            vram: Vram::new(),
            palette_ram: PaletteRam::new(),
            name_table_mirroring,
        }
    }
}
