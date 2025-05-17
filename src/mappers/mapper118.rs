use crate::mapper::*;

use crate::mappers::mmc3::mmc3::{Mapper004Mmc3, RegId};
use crate::mappers::mmc3::sharp_irq_state::SharpIrqState;

use super::mmc3::mmc3;

pub const LAYOUT: Layout = mmc3::LAYOUT.into_builder()
    // NameTableMirrorings in this mapper are set manually, rather than selected from MMC3's list.
    .name_table_mirrorings(&[])
    .build();

// TxSROM
pub struct Mapper118 {
    mmc3: Mapper004Mmc3,
}

impl Mapper for Mapper118 {
    fn write_register(&mut self, params: &mut MapperParams, cpu_address: u16, value: u8) {
        if matches!(cpu_address, 0xA000..=0xBFFF) && cpu_address % 2 == 0 {
            // Don't set NameTableMirroring from MMC3's standard list.
            return;
        }

        if matches!(cpu_address, 0x8000..=0x9FFF) && cpu_address % 2 == 1 {
            let selected_layout = params.chr_memory().layout_index();
            let selected_register = self.mmc3.selected_register_id();
            use NameTableQuadrant::*;
            let quadrants: &[_] = match (selected_layout, selected_register) {
                (0, RegId::Chr(C0)) => &[TopLeft, TopRight],
                (0, RegId::Chr(C1)) => &[BottomLeft, BottomRight],
                (1, RegId::Chr(C2)) => &[TopLeft],
                (1, RegId::Chr(C3)) => &[TopRight],
                (1, RegId::Chr(C4)) => &[BottomLeft],
                (1, RegId::Chr(C5)) => &[BottomRight],
                _ => &[],
            };

            let ciram_side = if value >> 7 == 0 { CiramSide::Left } else { CiramSide::Right };
            for quadrant in quadrants {
                params.set_name_table_quadrant(*quadrant, ciram_side);
            }
        }

        self.mmc3.write_register(params, cpu_address, value);
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

impl Mapper118 {
    pub fn new() -> Self {
        Self {
            mmc3: Mapper004Mmc3::new(Box::new(SharpIrqState::new())),
        }
    }
}