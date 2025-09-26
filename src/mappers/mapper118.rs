use crate::mapper::*;

use crate::mappers::mmc3::irq_state::IrqState;
use crate::mappers::mmc3::mmc3;


pub const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(512 * KIBIBYTE)
    .prg_layout(mmc3::PRG_WINDOWS_8000_SWITCHABLE)
    .prg_layout(mmc3::PRG_WINDOWS_C000_SWITCHABLE)
    .chr_rom_max_size(256 * KIBIBYTE)
    .chr_layout(mmc3::CHR_BIG_WINDOWS_FIRST)
    .chr_layout(mmc3::CHR_SMALL_WINDOWS_FIRST)
    // NameTableMirrorings in this mapper are set manually, rather than selected from MMC3's list.
    .name_table_mirrorings(&[])
    .read_write_statuses(mmc3::READ_WRITE_STATUSES)
    .build();

// TxSROM
pub struct Mapper118 {
    mmc3: mmc3::Mapper004Mmc3,
}

impl Mapper for Mapper118 {
    fn write_register(&mut self, mem: &mut Memory, addr: CpuAddress, value: u8) {
        if matches!(*addr, 0xA000..=0xBFFF) && addr.is_multiple_of(2) {
            // Don't set NameTableMirroring from MMC3's standard list.
            return;
        }

        if matches!(*addr, 0x8000..=0x9FFF) && *addr % 2 == 1 {
            let selected_layout = mem.chr_memory().layout_index();
            let selected_register = self.mmc3.selected_register_id();
            use NameTableQuadrant::*;
            let quadrants: &[_] = match (selected_layout, selected_register) {
                (0, mmc3::RegId::Chr(C0)) => &[TopLeft, TopRight],
                (0, mmc3::RegId::Chr(C1)) => &[BottomLeft, BottomRight],
                (1, mmc3::RegId::Chr(C2)) => &[TopLeft],
                (1, mmc3::RegId::Chr(C3)) => &[TopRight],
                (1, mmc3::RegId::Chr(C4)) => &[BottomLeft],
                (1, mmc3::RegId::Chr(C5)) => &[BottomRight],
                _ => &[],
            };

            let ciram_side = if value >> 7 == 0 { CiramSide::Left } else { CiramSide::Right };
            for quadrant in quadrants {
                mem.set_name_table_quadrant(*quadrant, ciram_side);
            }
        }

        self.mmc3.write_register(mem, addr, value);
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

impl Mapper118 {
    pub fn new() -> Self {
        Self {
            mmc3: mmc3::Mapper004Mmc3::new(IrqState::SHARP_IRQ_STATE),
        }
    }
}