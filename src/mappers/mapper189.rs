use crate::mapper::*;

use crate::mappers::mmc3::mmc3::{Mapper004Mmc3, RegId};
use crate::mappers::mmc3::irq_state::IrqState;

use super::mmc3::mmc3;

// FIXME: Just write out the full layout. No need to same a few lines of code.
pub const LAYOUT: Layout = mmc3::LAYOUT.into_builder_with_prg_layouts_cleared()
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::EMPTY),
        PrgWindow::new(0x8000, 0xFFFF, 32 * KIBIBYTE, PrgBank::ROM.switchable(P0)),
    ])
    .build();

// TXC-PT8154
pub struct Mapper189 {
    mmc3: Mapper004Mmc3,
}

impl Mapper for Mapper189 {
    fn write_register(&mut self, mem: &mut Memory, addr: CpuAddress, value: u8) {
        match (*addr, self.mmc3.selected_register_id()) {
            (0x4120..=0x7FFF, _) => {
                let bank_index = (value >> 4) | (value & 0b1111);
                mem.set_prg_register(P0, bank_index);
            }
            (0x8000..=0xBFFF, RegId::Prg(_)) if *addr % 2 == 1 => {
                // Do nothing here: PRG registers are not set by the standard MMC3 process.
            }
            _ => {
                // Most registers are standard MMC3.
                self.mmc3.write_register(mem, addr, value);
            }
        }
    }

    fn on_end_of_ppu_cycle(&mut self) {
        self.mmc3.on_end_of_ppu_cycle();
    }

    fn on_ppu_address_change(&mut self, mem: &mut Memory, address: PpuAddress) {
        self.mmc3.on_ppu_address_change(mem, address);
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}

impl Mapper189 {
    pub fn new() -> Self {
        Self {
            mmc3: Mapper004Mmc3::new(IrqState::SHARP_IRQ_STATE),
        }
    }
}
