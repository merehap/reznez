use crate::mapper::*;
use crate::mappers::vrc::vrc4;
use crate::mappers::mapper023_2;

const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(256 * KIBIBYTE)
    // Identical to VRC4, except there is bank-switchable ROM at 0x6000 instead.
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(S)),
        PrgWindow::new(0x8000, 0x9FFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P)),
        PrgWindow::new(0xA000, 0xBFFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(Q)),
        PrgWindow::new(0xC000, 0xDFFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(R)),
        PrgWindow::new(0xE000, 0xFFFF, 8 * KIBIBYTE, PrgBank::ROM.fixed_number(-1)),
    ])
    // CHR and mirroring is identical to VRC4.
    .chr_rom_max_size(512 * KIBIBYTE)
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x03FF, 1 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C)),
        ChrWindow::new(0x0400, 0x07FF, 1 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(D)),
        ChrWindow::new(0x0800, 0x0BFF, 1 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(E)),
        ChrWindow::new(0x0C00, 0x0FFF, 1 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(F)),
        ChrWindow::new(0x1000, 0x13FF, 1 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(G)),
        ChrWindow::new(0x1400, 0x17FF, 1 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(H)),
        ChrWindow::new(0x1800, 0x1BFF, 1 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(I)),
        ChrWindow::new(0x1C00, 0x1FFF, 1 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(J)),
    ])
    .name_table_mirrorings(&[
        NameTableMirroring::VERTICAL,
        NameTableMirroring::HORIZONTAL,
        NameTableMirroring::ONE_SCREEN_LEFT_BANK,
        NameTableMirroring::ONE_SCREEN_RIGHT_BANK,
    ])
    .build();

pub struct Mapper183 {
    vrc4e: vrc4::Vrc4,
}

impl Mapper for Mapper183 {
    fn write_register(&mut self, bus: &mut Bus, addr: CpuAddress, value: u8) {
        match *addr {
            0x6000..=0x7FFF => bus.set_prg_register(S, *addr & 0b1111),
            0x8800..=0x8803 => bus.set_prg_register(P, value & 0b0001_1111),
            0x9800..=0x9803 => bus.set_name_table_mirroring(value & 0b11),
            0xA000..=0xA003 => bus.set_prg_register(R, value & 0b0001_1111),
            0xA800..=0xA803 => bus.set_prg_register(Q, value & 0b0001_1111),
            0x8000..=0xA003 => { /* Ignore VRC4 PRG handling. This mapper does it differently. */ }
            0x0000..=0x5FFF | 0xA004..=0xFFFF => self.vrc4e.write_register(bus, addr, value),
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}

impl Mapper183 {
    pub fn new() -> Self {
        Self {
            vrc4e: mapper023_2::mapper023_2(),
        }
    }
}