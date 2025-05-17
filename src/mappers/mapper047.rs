use crate::mapper::*;
use crate::mappers::mmc3::mmc3;

use super::mmc3::sharp_irq_state::SharpIrqState;

const LAYOUT: Layout = mmc3::LAYOUT.into_builder()
    .prg_rom_max_size(256 * KIBIBYTE)
    .prg_rom_outer_bank_size(128 * KIBIBYTE)
    // Same PRG layouts as MMC3, except no RAM allowed.
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF, 8 * KIBIBYTE, PrgBank::EMPTY),
        PrgWindow::new(0x8000, 0x9FFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P0)),
        PrgWindow::new(0xA000, 0xBFFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P1)),
        PrgWindow::new(0xC000, 0xDFFF, 8 * KIBIBYTE, PrgBank::ROM.fixed_index(-2)),
        PrgWindow::new(0xE000, 0xFFFF, 8 * KIBIBYTE, PrgBank::ROM.fixed_index(-1)),
    ])
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF, 8 * KIBIBYTE, PrgBank::EMPTY),
        PrgWindow::new(0x8000, 0x9FFF, 8 * KIBIBYTE, PrgBank::ROM.fixed_index(-2)),
        PrgWindow::new(0xA000, 0xBFFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P1)),
        PrgWindow::new(0xC000, 0xDFFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P0)),
        PrgWindow::new(0xE000, 0xFFFF, 8 * KIBIBYTE, PrgBank::ROM.fixed_index(-1)),
    ])
    .chr_rom_max_size(256 * KIBIBYTE)
    .chr_rom_outer_bank_size(128 * KIBIBYTE)
    .build();

pub struct Mapper047 {
    mmc3: mmc3::Mapper004Mmc3,
}

impl Mapper for Mapper047 {
    fn write_register(&mut self, params: &mut MapperParams, cpu_address: u16, value: u8) {
        if matches!(cpu_address, 0x6000..=0x7FFF) {
            // S0 isn't hooked up to any window, but its value is still set by MMC3 and used for this mapper.
            if params.prg_memory().bank_registers().read_write_status(S0) == ReadWriteStatus::ReadWrite {
                let index = value & 1;
                params.set_prg_rom_outer_bank_index(index);
                params.set_chr_rom_outer_bank_index(index);
            }
        } else {
            self.mmc3.write_register(params, cpu_address, value);
        }
    }

    fn on_end_of_ppu_cycle(&mut self) {
        self.mmc3.on_end_of_ppu_cycle();
    }

    fn on_ppu_address_change(&mut self, params: &mut MapperParams, address: PpuAddress) {
        self.mmc3.on_ppu_address_change(params, address);
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}

impl Mapper047 {
    pub fn new() -> Self {
        Mapper047 {
            mmc3: mmc3::Mapper004Mmc3::new(Box::new(SharpIrqState::new())),
        }
    }
}