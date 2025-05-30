use crate::mapper::*;

use crate::mappers::mmc3::mmc3::{self, Mapper004Mmc3, RegId};
use crate::mappers::mmc3::sharp_irq_state::SharpIrqState;
use crate::memory::bank::bank::RomRamModeRegisterId;
use crate::memory::bank::bank_index::MemoryType;

pub const LAYOUT: Layout = mmc3::LAYOUT.into_builder_with_chr_layouts_cleared()
    .prg_rom_max_size(128 * KIBIBYTE)
    .chr_rom_max_size(64 * KIBIBYTE)
    // Same CHR layouts as standard MMC3, except the banks can switch between ROM and RAM memory spaces.
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x07FF, 2 * KIBIBYTE, ChrBank::ROM_RAM.switchable(C0).rom_ram_register(R0)),
        ChrWindow::new(0x0800, 0x0FFF, 2 * KIBIBYTE, ChrBank::ROM_RAM.switchable(C1).rom_ram_register(R1)),
        ChrWindow::new(0x1000, 0x13FF, 1 * KIBIBYTE, ChrBank::ROM_RAM.switchable(C2).rom_ram_register(R2)),
        ChrWindow::new(0x1400, 0x17FF, 1 * KIBIBYTE, ChrBank::ROM_RAM.switchable(C3).rom_ram_register(R3)),
        ChrWindow::new(0x1800, 0x1BFF, 1 * KIBIBYTE, ChrBank::ROM_RAM.switchable(C4).rom_ram_register(R4)),
        ChrWindow::new(0x1C00, 0x1FFF, 1 * KIBIBYTE, ChrBank::ROM_RAM.switchable(C5).rom_ram_register(R5)),
    ])
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x03FF, 1 * KIBIBYTE, ChrBank::ROM_RAM.switchable(C2).rom_ram_register(R2)),
        ChrWindow::new(0x0400, 0x07FF, 1 * KIBIBYTE, ChrBank::ROM_RAM.switchable(C3).rom_ram_register(R3)),
        ChrWindow::new(0x0800, 0x0BFF, 1 * KIBIBYTE, ChrBank::ROM_RAM.switchable(C4).rom_ram_register(R4)),
        ChrWindow::new(0x0C00, 0x0FFF, 1 * KIBIBYTE, ChrBank::ROM_RAM.switchable(C5).rom_ram_register(R5)),
        ChrWindow::new(0x1000, 0x17FF, 2 * KIBIBYTE, ChrBank::ROM_RAM.switchable(C0).rom_ram_register(R0)),
        ChrWindow::new(0x1800, 0x1FFF, 2 * KIBIBYTE, ChrBank::ROM_RAM.switchable(C1).rom_ram_register(R1)),
    ])
    .build();

const ROM_RAM_REGISTER_IDS: [RomRamModeRegisterId; 6] = [R0, R1, R2, R3, R4, R5];

// TQROM
pub struct Mapper119 {
    mmc3: Mapper004Mmc3,
}

impl Mapper for Mapper119 {
    fn write_register(&mut self, params: &mut MapperParams, cpu_address: u16, value: u8) {
        if matches!(cpu_address, 0x8001..=0x9FFF)
                && cpu_address % 2 == 1
                && let RegId::Chr(chr_id) = self.mmc3.selected_register_id() {
            let (use_ram, bank_index) = splitbits_named!(value, ".rbbbbbb");
            let rom_ram_reg_id = ROM_RAM_REGISTER_IDS[chr_id as usize];
            let memory_type = if use_ram { MemoryType::Ram } else { MemoryType::Rom };

            params.set_chr_register(chr_id, bank_index);
            params.set_rom_ram_mode(rom_ram_reg_id, memory_type);
        } else {
            // Use standard MMC3 behaviors.
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

impl Mapper119 {
    pub fn new() -> Self {
        Self {
            mmc3: Mapper004Mmc3::new(Box::new(SharpIrqState::new())),
        }
    }
}