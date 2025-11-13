use crate::mapper::*;
use crate::mappers;
use crate::mappers::mmc3::mmc3;

pub const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(256 * KIBIBYTE)
    // Can be set to 128KiB through writing to the outer bank select register (0x6000).
    .prg_rom_outer_bank_size(64 * KIBIBYTE)
    // The PRG layouts are the same as MMC3, except they can't have RAM.
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF, 8 * KIBIBYTE, PrgBank::ABSENT),
        PrgWindow::new(0x8000, 0x9FFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P0)),
        PrgWindow::new(0xA000, 0xBFFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P1)),
        PrgWindow::new(0xC000, 0xDFFF, 8 * KIBIBYTE, PrgBank::ROM.fixed_index(-2)),
        PrgWindow::new(0xE000, 0xFFFF, 8 * KIBIBYTE, PrgBank::ROM.fixed_index(-1)),
    ])
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF, 8 * KIBIBYTE, PrgBank::ABSENT),
        PrgWindow::new(0x8000, 0x9FFF, 8 * KIBIBYTE, PrgBank::ROM.fixed_index(-2)),
        PrgWindow::new(0xA000, 0xBFFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P1)),
        PrgWindow::new(0xC000, 0xDFFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P0)),
        PrgWindow::new(0xE000, 0xFFFF, 8 * KIBIBYTE, PrgBank::ROM.fixed_index(-1)),
    ])
    .chr_rom_max_size(256 * KIBIBYTE)
    .chr_rom_outer_bank_size(128 * KIBIBYTE)
    .chr_layout(mmc3::CHR_BIG_WINDOWS_FIRST)
    .chr_layout(mmc3::CHR_SMALL_WINDOWS_FIRST)
    .name_table_mirrorings(mmc3::NAME_TABLE_MIRRORINGS)
    .build();

// Super Mario Bros. + Tetris + Nintendo World Cup
// FIXME: Untested since PAL PPUs aren't supported yet (only NTSC)
pub struct Mapper037 {
    mmc3: mmc3::Mapper004Mmc3,
}

impl Mapper for Mapper037 {
    fn write_register(&mut self, mem: &mut Memory, addr: CpuAddress, value: u8) {
        // MMC3 is still setting W0 WriteStatus to Enabled/Disabled,
        // even though this mapper substitutes in a layout that doesn't use W0.
        if matches!(*addr, 0x6000..=0x7FFF) && mem.prg_memory.bank_registers().write_status(W0) == WriteStatus::Enabled {
            mem.chr_memory.set_chr_rom_outer_bank_number((value >> 2) & 1);

            let (new_prg_outer_bank_size, new_prg_outer_bank_number) = match value & 0b111 {
                0..=2 => ( 64 * KIBIBYTE, 0), // 0x00000 to 0x0FFFF
                3     => ( 64 * KIBIBYTE, 1), // 0x10000 to 0x1FFFF
                4..=6 => (128 * KIBIBYTE, 1), // 0x20000 to 0x3FFFF
                7     => ( 64 * KIBIBYTE, 3), // 0x30000 to 0x3FFFF
                _ => unimplemented!(),
            };
            mem.prg_memory.set_prg_rom_outer_bank_size(new_prg_outer_bank_size);
            mem.prg_memory.set_prg_rom_outer_bank_number(new_prg_outer_bank_number);
        }

        self.mmc3.write_register(mem, addr, value);
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}

impl Mapper037 {
    pub fn new() -> Self {
        Self { mmc3: mappers::mapper004_0::mapper004_0() }
    }
}