use crate::mapper::*;

use crate::mappers::mmc3::mmc3;
use crate::mappers::mmc3::irq_state::Mmc3IrqState;

pub const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(256 * KIBIBYTE)
    // Single layout with a 32KiB window instead of normal MMC3's two layouts with 8KiB windows.
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::ABSENT),
        PrgWindow::new(0x8000, 0xFFFF, 32 * KIBIBYTE, PrgBank::ROM.switchable(P0)),
    ])
    .chr_rom_max_size(256 * KIBIBYTE)
    .chr_layout(mmc3::CHR_BIG_WINDOWS_FIRST)
    .chr_layout(mmc3::CHR_SMALL_WINDOWS_FIRST)
    .name_table_mirrorings(mmc3::NAME_TABLE_MIRRORINGS)
    .read_write_statuses(mmc3::READ_WRITE_STATUSES)
    .build();

// TXC-PT8154
pub struct Mapper189 {
    mmc3: mmc3::Mapper004Mmc3,
}

impl Mapper for Mapper189 {
    fn write_register(&mut self, mem: &mut Memory, addr: CpuAddress, value: u8) {
        match (*addr, self.mmc3.selected_register_id()) {
            (0x4120..=0x7FFF, _) => {
                let bank_number = (value >> 4) | (value & 0b1111);
                mem.set_prg_register(P0, bank_number);
            }
            (0x8000..=0xBFFF, mmc3::RegId::Prg(_)) if *addr % 2 == 1 => {
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
            mmc3: mmc3::Mapper004Mmc3::new(Mmc3IrqState::SHARP_IRQ_STATE),
        }
    }
}
