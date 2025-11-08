use crate::mapper::*;
use crate::mappers::mmc3::mmc3;
use crate::mappers::mmc3::irq_state::Mmc3IrqState;

use super::mmc3::mmc3::RegId;

const LAYOUT: Layout = Layout::builder()
    // Switchable 0x8000
    .prg_rom_max_size(512 * KIBIBYTE)
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x6FFF, 4 * KIBIBYTE, PrgBank::ABSENT),
        PrgWindow::new(0x7000, 0x71FF, KIBIBYTE / 2, PrgBank::WORK_RAM.fixed_index(0).read_write_status(R0, W0)),
        PrgWindow::new(0x7200, 0x73FF, KIBIBYTE / 2, PrgBank::WORK_RAM.fixed_index(1).read_write_status(R1, W1)),
        PrgWindow::new(0x7400, 0x75FF, KIBIBYTE / 2, PrgBank::WORK_RAM.fixed_index(0).read_write_status(R0, W0)),
        PrgWindow::new(0x7600, 0x77FF, KIBIBYTE / 2, PrgBank::WORK_RAM.fixed_index(1).read_write_status(R1, W1)),
        PrgWindow::new(0x7800, 0x79FF, KIBIBYTE / 2, PrgBank::WORK_RAM.fixed_index(0).read_write_status(R0, W0)),
        PrgWindow::new(0x7A00, 0x7BFF, KIBIBYTE / 2, PrgBank::WORK_RAM.fixed_index(1).read_write_status(R1, W1)),
        PrgWindow::new(0x7C00, 0x7DFF, KIBIBYTE / 2, PrgBank::WORK_RAM.fixed_index(0).read_write_status(R0, W0)),
        PrgWindow::new(0x7E00, 0x7FFF, KIBIBYTE / 2, PrgBank::WORK_RAM.fixed_index(1).read_write_status(R1, W1)),
        PrgWindow::new(0x8000, 0x9FFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P0)),
        PrgWindow::new(0xA000, 0xBFFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P1)),
        PrgWindow::new(0xC000, 0xDFFF, 8 * KIBIBYTE, PrgBank::ROM.fixed_index(-2)),
        PrgWindow::new(0xE000, 0xFFFF, 8 * KIBIBYTE, PrgBank::ROM.fixed_index(-1)),
    ])
    // Switchable 0xC000
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x6FFF, 4 * KIBIBYTE, PrgBank::ABSENT),
        PrgWindow::new(0x7000, 0x71FF, KIBIBYTE / 2, PrgBank::WORK_RAM.fixed_index(0).read_write_status(R0, W0)),
        PrgWindow::new(0x7200, 0x73FF, KIBIBYTE / 2, PrgBank::WORK_RAM.fixed_index(1).read_write_status(R1, W1)),
        PrgWindow::new(0x7400, 0x75FF, KIBIBYTE / 2, PrgBank::WORK_RAM.fixed_index(0).read_write_status(R0, W0)),
        PrgWindow::new(0x7600, 0x77FF, KIBIBYTE / 2, PrgBank::WORK_RAM.fixed_index(1).read_write_status(R1, W1)),
        PrgWindow::new(0x7800, 0x79FF, KIBIBYTE / 2, PrgBank::WORK_RAM.fixed_index(0).read_write_status(R0, W0)),
        PrgWindow::new(0x7A00, 0x7BFF, KIBIBYTE / 2, PrgBank::WORK_RAM.fixed_index(1).read_write_status(R1, W1)),
        PrgWindow::new(0x7C00, 0x7DFF, KIBIBYTE / 2, PrgBank::WORK_RAM.fixed_index(0).read_write_status(R0, W0)),
        PrgWindow::new(0x7E00, 0x7FFF, KIBIBYTE / 2, PrgBank::WORK_RAM.fixed_index(1).read_write_status(R1, W1)),
        PrgWindow::new(0x8000, 0x9FFF, 8 * KIBIBYTE, PrgBank::ROM.fixed_index(-2)),
        PrgWindow::new(0xA000, 0xBFFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P1)),
        PrgWindow::new(0xC000, 0xDFFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P0)),
        PrgWindow::new(0xE000, 0xFFFF, 8 * KIBIBYTE, PrgBank::ROM.fixed_index(-1)),
    ])
    .chr_rom_max_size(256 * KIBIBYTE)
    .chr_layout(mmc3::CHR_BIG_WINDOWS_FIRST)
    .chr_layout(mmc3::CHR_SMALL_WINDOWS_FIRST)
    .name_table_mirrorings(&[
        NameTableMirroring::VERTICAL,
        NameTableMirroring::HORIZONTAL,
    ])
    .build();

// MMC6. Similar to MMC3 with Rev A IRQs, but with Work RAM protection.
// TODO: Support VS System (and its 4-screen mirroring).
pub struct Mapper004_1 {
    selected_register_id: RegId,
    irq_state: Mmc3IrqState,
    prg_ram_enabled: bool,
}

impl Mapper for Mapper004_1 {
    fn write_register(&mut self, mem: &mut Memory, addr: CpuAddress, value: u8) {
        let is_even_address = addr.is_multiple_of(2);
        match (*addr, is_even_address) {
            (0x0000..=0x401F, _) => unreachable!(),
            (0x4020..=0x7FFF, _) => { /* Do nothing. */ }
            (0x8000..=0x9FFF, true ) => {
                let fields = splitbits!(value, "cpr..bbb");
                mem.set_chr_layout(fields.c as u8);
                mem.set_prg_layout(fields.p as u8);
                self.prg_ram_enabled = fields.r;
                if !self.prg_ram_enabled {
                    mem.set_reads_enabled(R0, false);
                    mem.set_reads_enabled(R1, false);
                    mem.set_writes_enabled(W0, false);
                    mem.set_writes_enabled(W1, false);
                }

                self.selected_register_id = mmc3::BANK_NUMBER_REGISTER_IDS[fields.b as usize];
            }
            (0x8000..=0x9FFF, false) => {
                match self.selected_register_id {
                    RegId::Chr(cx) => mem.set_chr_register(cx, value),
                    RegId::Prg(px) => mem.set_prg_register(px, value),
                }
            },
            (0xA000..=0xBFFF, true ) => {
                // Hard-coded 4-screen mirroring cannot be overridden.
                if mem.name_table_mirroring().is_vertical() || mem.name_table_mirroring().is_horizontal() {
                    mem.set_name_table_mirroring(value & 1);
                }
            }
            (0xA000..=0xBFFF, false) => {
                if !self.prg_ram_enabled {
                    // Keep reads and writes disabled for R0/W0 and R1/W1.
                    return
                }

                let (enable_7200, writeable_7200, enable_7000, writeable_7000) = splitbits_named!(value, "ewfx ....");
                mem.set_writes_enabled(W0, enable_7000 && writeable_7000);
                mem.set_writes_enabled(W1, enable_7200 && writeable_7200);

                mem.set_reads_enabled(R0, enable_7000);
                mem.set_reads_enabled(R1, enable_7200);

                if !enable_7000 && enable_7200  {
                    // Overwrite/ignore the value that R0 was set to above.
                    mem.set_read_zeroes(R0);
                }

                if !enable_7200 && enable_7000 {
                    // Overwrite/ignore the value that R1 was set to above.
                    mem.set_read_zeroes(R1);
                }
            }
            (0xC000..=0xDFFF, true ) => self.irq_state.set_counter_reload_value(value),
            (0xC000..=0xDFFF, false) => self.irq_state.reload_counter(),
            (0xE000..=0xFFFF, true ) => self.irq_state.disable(mem),
            (0xE000..=0xFFFF, false) => self.irq_state.enable(),
        }
    }

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

impl Mapper004_1 {
    pub fn new() -> Self {
        Self {
            selected_register_id: RegId::Chr(C0),
            irq_state: Mmc3IrqState::REV_A_IRQ_STATE,
            prg_ram_enabled: false,
        }
    }
}