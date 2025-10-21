use crate::mapper::*;

use super::mmc3::irq_state::Mmc3IrqState;

const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(2048 * KIBIBYTE)
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF, 8 * KIBIBYTE, PrgBank::ABSENT),
        PrgWindow::new(0x8000, 0x9FFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P0)),
        PrgWindow::new(0xA000, 0xBFFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P1)),
        PrgWindow::new(0xC000, 0xDFFF, 8 * KIBIBYTE, PrgBank::ROM.fixed_index(-2)),
        PrgWindow::new(0xE000, 0xFFFF, 8 * KIBIBYTE, PrgBank::ROM.fixed_index(-1)),
    ])
    .chr_rom_max_size(512 * KIBIBYTE)
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x07FF, 2 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C0)),
        ChrWindow::new(0x0800, 0x0FFF, 2 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C1)),
        ChrWindow::new(0x1000, 0x13FF, 1 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C2)),
        ChrWindow::new(0x1400, 0x17FF, 1 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C3)),
        ChrWindow::new(0x1800, 0x1BFF, 1 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C4)),
        ChrWindow::new(0x1C00, 0x1FFF, 1 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C5)),
    ])
    .do_not_align_large_chr_windows()
    .name_table_mirrorings(&[
        NameTableMirroring::VERTICAL,
        NameTableMirroring::HORIZONTAL,
    ])
    .build();

// Taito's TC0690
pub struct Mapper048 {
    irq_state: Mmc3IrqState,
}

impl Mapper for Mapper048 {
    fn write_register(&mut self, mem: &mut Memory, addr: CpuAddress, value: u8) {
        let bank_number = u16::from(value);
        match *addr & 0xE003 {
            0x8000 => mem.set_prg_register(P0, bank_number),
            0x8001 => mem.set_prg_register(P1, bank_number),
            0x8002 => mem.set_chr_register(C0, 2 * bank_number),
            0x8003 => mem.set_chr_register(C1, 2 * bank_number),
            0xA000 => mem.set_chr_register(C2, bank_number),
            0xA001 => mem.set_chr_register(C3, bank_number),
            0xA002 => mem.set_chr_register(C4, bank_number),
            0xA003 => mem.set_chr_register(C5, bank_number),
            0xC000 => self.irq_state.set_counter_reload_value(value ^ 0xFF),
            0xC001 => self.irq_state.reload_counter(),
            0xC002 => self.irq_state.enable(),
            0xC003 => self.irq_state.disable(mem),
            0xE000 => mem.set_name_table_mirroring((value << 1) >> 7),
            _ => { /* Do nothing. */ }
        }
    }

    // FIXME: This doesn't match the CPU cycle-based suppression needed for this mapper.
    fn on_end_of_ppu_cycle(&mut self) {
        self.irq_state.decrement_suppression_cycle_count();
    }

    fn on_ppu_address_change(&mut self, mem: &mut Memory, address: PpuAddress) {
        self.irq_state.tick_counter(mem, address);
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}

impl Mapper048 {
    pub fn new() -> Self {
        Self { irq_state: Mmc3IrqState::SHARP_IRQ_STATE }
    }
}
