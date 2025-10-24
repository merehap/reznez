use crate::mapper::*;

use crate::mappers::mmc3::mmc3;
use crate::mappers::mmc3::irq_state::Mmc3IrqState;
use crate::memory::bank::bank::{ChrSource, ChrSourceRegisterId};

pub const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(128 * KIBIBYTE)
    .prg_layout(mmc3::PRG_WINDOWS_8000_SWITCHABLE)
    .prg_layout(mmc3::PRG_WINDOWS_C000_SWITCHABLE)
    .chr_rom_max_size(64 * KIBIBYTE)
    // Same CHR layouts as standard MMC3, except the banks can switch between ROM and RAM memory spaces.
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x07FF, 2 * KIBIBYTE, ChrBank::SWITCHABLE_SOURCE.switchable(C0).chr_source(CS0)),
        ChrWindow::new(0x0800, 0x0FFF, 2 * KIBIBYTE, ChrBank::SWITCHABLE_SOURCE.switchable(C1).chr_source(CS1)),
        ChrWindow::new(0x1000, 0x13FF, 1 * KIBIBYTE, ChrBank::SWITCHABLE_SOURCE.switchable(C2).chr_source(CS2)),
        ChrWindow::new(0x1400, 0x17FF, 1 * KIBIBYTE, ChrBank::SWITCHABLE_SOURCE.switchable(C3).chr_source(CS3)),
        ChrWindow::new(0x1800, 0x1BFF, 1 * KIBIBYTE, ChrBank::SWITCHABLE_SOURCE.switchable(C4).chr_source(CS4)),
        ChrWindow::new(0x1C00, 0x1FFF, 1 * KIBIBYTE, ChrBank::SWITCHABLE_SOURCE.switchable(C5).chr_source(CS5)),
    ])
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x03FF, 1 * KIBIBYTE, ChrBank::SWITCHABLE_SOURCE.switchable(C2).chr_source(CS2)),
        ChrWindow::new(0x0400, 0x07FF, 1 * KIBIBYTE, ChrBank::SWITCHABLE_SOURCE.switchable(C3).chr_source(CS3)),
        ChrWindow::new(0x0800, 0x0BFF, 1 * KIBIBYTE, ChrBank::SWITCHABLE_SOURCE.switchable(C4).chr_source(CS4)),
        ChrWindow::new(0x0C00, 0x0FFF, 1 * KIBIBYTE, ChrBank::SWITCHABLE_SOURCE.switchable(C5).chr_source(CS5)),
        ChrWindow::new(0x1000, 0x17FF, 2 * KIBIBYTE, ChrBank::SWITCHABLE_SOURCE.switchable(C0).chr_source(CS0)),
        ChrWindow::new(0x1800, 0x1FFF, 2 * KIBIBYTE, ChrBank::SWITCHABLE_SOURCE.switchable(C1).chr_source(CS1)),
    ])
    .name_table_mirrorings(mmc3::NAME_TABLE_MIRRORINGS)
    .build();

const ROM_RAM_REGISTER_IDS: [ChrSourceRegisterId; 6] = [CS0, CS1, CS2, CS3, CS4, CS5];

// TQROM
pub struct Mapper119 {
    mmc3: mmc3::Mapper004Mmc3,
}

impl Mapper for Mapper119 {
    fn write_register(&mut self, mem: &mut Memory, addr: CpuAddress, value: u8) {
        if matches!(*addr, 0x8001..=0x9FFF)
                && !addr.is_multiple_of(2)
                && let mmc3::RegId::Chr(chr_id) = self.mmc3.selected_register_id() {

            let fields = splitbits!(value, ".mcccccc");
            mem.set_chr_register(chr_id, fields.c);
            let rom_ram_reg_id = ROM_RAM_REGISTER_IDS[chr_id as usize];
            let chr_source = [ChrSource::Rom, ChrSource::WorkRam][fields.m as usize];
            mem.set_chr_source(rom_ram_reg_id, chr_source);
        } else {
            // Use standard MMC3 behaviors.
            self.mmc3.write_register(mem, addr, value);
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

impl Mapper119 {
    pub fn new() -> Self {
        Self {
            mmc3: mmc3::Mapper004Mmc3::new(Mmc3IrqState::SHARP_IRQ_STATE),
        }
    }
}