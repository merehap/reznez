pub struct PpuRegisters<'a> {
    regs: &'a [u8; 8],
    oam_dma: &'a u8,
}

impl <'a> PpuRegisters<'a> {
    pub fn from_mem(regs: &'a [u8; 8], oam_dma: &'a u8) -> PpuRegisters<'a> {
        PpuRegisters {regs, oam_dma}
    }

    pub fn oam_addr(&self) -> u8 {
        self.regs[3]
    }

    pub fn oam_data(&self) -> u8 {
        self.regs[4]
    }

    pub fn ppu_scroll(&self) -> u8 {
        self.regs[5]
    }

    pub fn ppu_addr(&self) -> u8 {
        self.regs[6]
    }

    pub fn ppu_data(&self) -> u8 {
        self.regs[7]
    }

    pub fn oam_dma(&self) -> u8 {
        *self.oam_dma
    }
}
