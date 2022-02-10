use crate::memory::ppu::palette_ram::PaletteRam;
use crate::memory::ppu::vram::Vram;

pub struct PpuInternalRam {
    pub vram: Vram,
    pub palette_ram: PaletteRam,
}

impl PpuInternalRam {
    pub fn new() -> PpuInternalRam {
        PpuInternalRam {
            vram: Vram::new(),
            palette_ram: PaletteRam::new(),
        }
    }
}
