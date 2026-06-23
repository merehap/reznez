use crate::mapper::mapper::*;
use crate::mapper::mappers;
use crate::mapper::mappers::mmc3::mmc3;

pub const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(256 * KIBIBYTE)
    // The PRG layouts are the same as MMC3, except they can't have RAM, and the outer bank size can vary.
    // $00000-$0FFFF (64kiB outer bank)
    // $10000-$1FFFF (64kiB outer bank)
    // $30000-$3FFFF (64kiB outer bank)
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF, 8 * KIBIBYTE, Prg::ABSENT),
        PrgWindow::new(0x8000, 0x9FFF, 8 * KIBIBYTE, Prg::ROM).rom_address_template("o₀₁o₀₀p₀₂p₀₁p₀₀a₁₂a₁₁a₁₀a₀₉a₀₈a₀₇a₀₆a₀₅a₀₄a₀₃a₀₂a₀₁a₀₀"),
        PrgWindow::new(0xA000, 0xBFFF, 8 * KIBIBYTE, Prg::ROM).rom_address_template("o₀₁o₀₀q₀₂q₀₁q₀₀a₁₂a₁₁a₁₀a₀₉a₀₈a₀₇a₀₆a₀₅a₀₄a₀₃a₀₂a₀₁a₀₀"),
        PrgWindow::new(0xC000, 0xDFFF, 8 * KIBIBYTE, Prg::ROM).rom_address_template("o₀₁o₀₀1₀₂1₀₁0₀₀a₁₂a₁₁a₁₀a₀₉a₀₈a₀₇a₀₆a₀₅a₀₄a₀₃a₀₂a₀₁a₀₀"),
        PrgWindow::new(0xE000, 0xFFFF, 8 * KIBIBYTE, Prg::ROM).rom_address_template("o₀₁o₀₀1₀₂1₀₁1₀₀a₁₂a₁₁a₁₀a₀₉a₀₈a₀₇a₀₆a₀₅a₀₄a₀₃a₀₂a₀₁a₀₀"),
    ])
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF, 8 * KIBIBYTE, Prg::ABSENT),
        PrgWindow::new(0x8000, 0x9FFF, 8 * KIBIBYTE, Prg::ROM).rom_address_template("o₀₁o₀₀1₀₂1₀₁0₀₀a₁₂a₁₁a₁₀a₀₉a₀₈a₀₇a₀₆a₀₅a₀₄a₀₃a₀₂a₀₁a₀₀"),
        PrgWindow::new(0xA000, 0xBFFF, 8 * KIBIBYTE, Prg::ROM).rom_address_template("o₀₁o₀₀p₀₂p₀₁p₀₀a₁₂a₁₁a₁₀a₀₉a₀₈a₀₇a₀₆a₀₅a₀₄a₀₃a₀₂a₀₁a₀₀"),
        PrgWindow::new(0xC000, 0xDFFF, 8 * KIBIBYTE, Prg::ROM).rom_address_template("o₀₁o₀₀q₀₂q₀₁q₀₀a₁₂a₁₁a₁₀a₀₉a₀₈a₀₇a₀₆a₀₅a₀₄a₀₃a₀₂a₀₁a₀₀"),
        PrgWindow::new(0xE000, 0xFFFF, 8 * KIBIBYTE, Prg::ROM).rom_address_template("o₀₁o₀₀1₀₂1₀₁1₀₀a₁₂a₁₁a₁₀a₀₉a₀₈a₀₇a₀₆a₀₅a₀₄a₀₃a₀₂a₀₁a₀₀"),
    ])
    // $20000-$3FFFF (128kiB outer bank)
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF, 8 * KIBIBYTE, Prg::ABSENT),
        PrgWindow::new(0x8000, 0x9FFF, 8 * KIBIBYTE, Prg::ROM).rom_address_template("o₀₁p₀₃p₀₂p₀₁p₀₀a₁₂a₁₁a₁₀a₀₉a₀₈a₀₇a₀₆a₀₅a₀₄a₀₃a₀₂a₀₁a₀₀"),
        PrgWindow::new(0xA000, 0xBFFF, 8 * KIBIBYTE, Prg::ROM).rom_address_template("o₀₁q₀₃q₀₂q₀₁q₀₀a₁₂a₁₁a₁₀a₀₉a₀₈a₀₇a₀₆a₀₅a₀₄a₀₃a₀₂a₀₁a₀₀"),
        PrgWindow::new(0xC000, 0xDFFF, 8 * KIBIBYTE, Prg::ROM).rom_address_template("o₀₁1₀₃1₀₂1₀₁0₀₀a₁₂a₁₁a₁₀a₀₉a₀₈a₀₇a₀₆a₀₅a₀₄a₀₃a₀₂a₀₁a₀₀"),
        PrgWindow::new(0xE000, 0xFFFF, 8 * KIBIBYTE, Prg::ROM).rom_address_template("o₀₁1₀₃1₀₂1₀₁1₀₀a₁₂a₁₁a₁₀a₀₉a₀₈a₀₇a₀₆a₀₅a₀₄a₀₃a₀₂a₀₁a₀₀"),
    ])
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF, 8 * KIBIBYTE, Prg::ABSENT),
        PrgWindow::new(0x8000, 0x9FFF, 8 * KIBIBYTE, Prg::ROM).rom_address_template("o₀₁1₀₃1₀₂1₀₁0₀₀a₁₂a₁₁a₁₀a₀₉a₀₈a₀₇a₀₆a₀₅a₀₄a₀₃a₀₂a₀₁a₀₀"),
        PrgWindow::new(0xA000, 0xBFFF, 8 * KIBIBYTE, Prg::ROM).rom_address_template("o₀₁p₀₃p₀₂p₀₁p₀₀a₁₂a₁₁a₁₀a₀₉a₀₈a₀₇a₀₆a₀₅a₀₄a₀₃a₀₂a₀₁a₀₀"),
        PrgWindow::new(0xC000, 0xDFFF, 8 * KIBIBYTE, Prg::ROM).rom_address_template("o₀₁q₀₃q₀₂q₀₁q₀₀a₁₂a₁₁a₁₀a₀₉a₀₈a₀₇a₀₆a₀₅a₀₄a₀₃a₀₂a₀₁a₀₀"),
        PrgWindow::new(0xE000, 0xFFFF, 8 * KIBIBYTE, Prg::ROM).rom_address_template("o₀₁1₀₃1₀₂1₀₁1₀₀a₁₂a₁₁a₁₀a₀₉a₀₈a₀₇a₀₆a₀₅a₀₄a₀₃a₀₂a₀₁a₀₀"),
    ])
    .chr_rom_max_size(256 * KIBIBYTE)
    .chr_rom_outer_bank_size(128 * KIBIBYTE)
    .chr_layout(mmc3::CHR_BIG_WINDOWS_FIRST)
    .chr_layout(mmc3::CHR_SMALL_WINDOWS_FIRST)
    .name_table_mirrorings(mmc3::NAME_TABLE_MIRRORINGS)
    .build();

// Super Mario Bros. + Tetris + Nintendo World Cup
// FIXME: Graphical glitches on Nintendo World Cup.
pub struct Mapper037 {
    mmc3: mmc3::Mapper004Mmc3,
}

impl Mapper for Mapper037 {
    fn write_register(&mut self, bus: &mut Bus, addr: CpuAddress, value: u8) {
        // MMC3 is still setting W0 WriteStatus to Enabled/Disabled,
        // even though this mapper substitutes in a layout that doesn't use W0.
        if matches!(*addr, 0x6000..=0x7FFF) && bus.prg_memory.bank_registers().write_status(WS0) == WriteStatus::Enabled {
            bus.chr_memory.set_rom_outer_bank_number((value >> 2) & 1);

            let outer_bank_numbers = [0, 0, 0, 1, 2, 2, 2, 3];
            let outer_bank_number = outer_bank_numbers[usize::from(value & 0b111)];
            bus.set_prg_rom_outer_bank_number(outer_bank_number);
        }

        self.mmc3.write_register(bus, addr, value);

        let prg_outer_bank_number = bus.prg_rom_outer_bank_number();
        bus.update_effective_prg_layout_index(|base_index| {
            if prg_outer_bank_number == 2 {
                // Use the 128kiB outer bank layouts.
                base_index | 0b10
            } else {
                // Use the 64kiB outer bank layouts.
                base_index
            }
        });
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
