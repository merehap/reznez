use crate::ppu::name_table_number::NameTableNumber;
use crate::ppu::pattern_table::PatternTableSide;
use crate::util::get_bit;

pub struct PpuRegisters<'a> {
    regs: &'a [u8; 8],
    oam_dma: &'a u8,
}

impl <'a> PpuRegisters<'a> {
    pub fn from_mem(regs: &'a [u8; 8], oam_dma: &'a u8) -> PpuRegisters<'a> {
        PpuRegisters {regs, oam_dma}
    }

    pub fn vblank_nmi(&self) -> VBlankNmi {
        if get_bit(self.regs[0], 0) {VBlankNmi::On} else {VBlankNmi::Off}
    }

    pub fn ext_pin_role(&self) -> ExtPinRole {
        if get_bit(self.regs[0], 1) {ExtPinRole::Write} else {ExtPinRole::Read}
    }

    pub fn large_sprites(&self) -> SpriteSize {
        if get_bit(self.regs[0], 2) {SpriteSize::Wide} else {SpriteSize::Normal}
    }

    pub fn background_table_side(&self) -> PatternTableSide {
        if get_bit(self.regs[0], 3) {
            PatternTableSide::Right
        } else {
            PatternTableSide::Left
        }
    }

    pub fn sprite_table_side(&self) -> PatternTableSide {
        if get_bit(self.regs[0], 4) {
            PatternTableSide::Right
        } else {
            PatternTableSide::Left
        }
    }

    pub fn vram_address_increment(&self) -> VramAddressIncrement {
        if get_bit(self.regs[0], 5) {
            VramAddressIncrement::Down
        } else {
            VramAddressIncrement::Right
        }
    }

    pub fn name_table_number(&self) -> NameTableNumber {
        match (get_bit(self.regs[0], 6), get_bit(self.regs[0], 7)) {
            (false, false) => NameTableNumber::Zero,
            (false, true ) => NameTableNumber::One,
            (true , false) => NameTableNumber::Two,
            (true , true ) => NameTableNumber::Three,
        }
    }

    pub fn emphasize_blue(&self) -> bool {
        get_bit(self.regs[1], 0)
    }

    pub fn emphasize_green(&self) -> bool {
        get_bit(self.regs[1], 1)
    }

    pub fn emphasize_red(&self) -> bool {
        get_bit(self.regs[1], 2)
    }

    pub fn sprites_enabled(&self) -> bool {
        get_bit(self.regs[1], 3)
    }

    pub fn background_enabled(&self) -> bool {
        get_bit(self.regs[1], 4)
    }

    pub fn left_column_sprites_enabled(&self) -> bool {
        get_bit(self.regs[1], 5)
    }

    pub fn left_column_background_enabled(&self) -> bool {
        get_bit(self.regs[1], 6)
    }

    pub fn greyscale_enabled(&self) -> bool {
        get_bit(self.regs[1], 7)
    }

    pub fn vblank_active(&self) -> bool {
        get_bit(self.regs[2], 0)
    }

    pub fn sprite0_hit(&self) -> bool {
        get_bit(self.regs[2], 1)
    }

    pub fn sprite_overflow(&self) -> bool {
        get_bit(self.regs[2], 2)
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

pub enum VBlankNmi {
    Off,
    On,
}

pub enum ExtPinRole {
    Read,
    Write,
}

pub enum SpriteSize {
    Normal,
    Wide,
}

pub enum VramAddressIncrement {
    Right,
    Down,
}
