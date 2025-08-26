use crate::mapper::*;
use crate::mappers::mmc3::mmc3;
use crate::mappers::mmc3::irq_state::IrqState;
use crate::mappers::mmc3::rev_a_irq_state::RevAIrqState;

use super::mmc3::mmc3::RegId;

const LAYOUT: Layout = Layout::builder()
    // Switchable 0x8000
    .prg_rom_max_size(512 * KIBIBYTE)
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x6FFF, 4 * KIBIBYTE, PrgBank::EMPTY),
        PrgWindow::new(0x7000, 0x71FF, KIBIBYTE / 2, PrgBank::WORK_RAM.fixed_index(0).status_register(S0)),
        PrgWindow::new(0x7200, 0x73FF, KIBIBYTE / 2, PrgBank::WORK_RAM.fixed_index(1).status_register(S1)),
        PrgWindow::new(0x7400, 0x75FF, KIBIBYTE / 2, PrgBank::WORK_RAM.fixed_index(0).status_register(S0)),
        PrgWindow::new(0x7600, 0x77FF, KIBIBYTE / 2, PrgBank::WORK_RAM.fixed_index(1).status_register(S1)),
        PrgWindow::new(0x7800, 0x79FF, KIBIBYTE / 2, PrgBank::WORK_RAM.fixed_index(0).status_register(S0)),
        PrgWindow::new(0x7A00, 0x7BFF, KIBIBYTE / 2, PrgBank::WORK_RAM.fixed_index(1).status_register(S1)),
        PrgWindow::new(0x7C00, 0x7DFF, KIBIBYTE / 2, PrgBank::WORK_RAM.fixed_index(0).status_register(S0)),
        PrgWindow::new(0x7E00, 0x7FFF, KIBIBYTE / 2, PrgBank::WORK_RAM.fixed_index(1).status_register(S1)),
        PrgWindow::new(0x8000, 0x9FFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P0)),
        PrgWindow::new(0xA000, 0xBFFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P1)),
        PrgWindow::new(0xC000, 0xDFFF, 8 * KIBIBYTE, PrgBank::ROM.fixed_index(-2)),
        PrgWindow::new(0xE000, 0xFFFF, 8 * KIBIBYTE, PrgBank::ROM.fixed_index(-1)),
    ])
    // Switchable 0xC000
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x6FFF, 4 * KIBIBYTE, PrgBank::EMPTY),
        PrgWindow::new(0x7000, 0x71FF, KIBIBYTE / 2, PrgBank::WORK_RAM.fixed_index(0).status_register(S0)),
        PrgWindow::new(0x7200, 0x73FF, KIBIBYTE / 2, PrgBank::WORK_RAM.fixed_index(1).status_register(S1)),
        PrgWindow::new(0x7400, 0x75FF, KIBIBYTE / 2, PrgBank::WORK_RAM.fixed_index(0).status_register(S0)),
        PrgWindow::new(0x7600, 0x77FF, KIBIBYTE / 2, PrgBank::WORK_RAM.fixed_index(1).status_register(S1)),
        PrgWindow::new(0x7800, 0x79FF, KIBIBYTE / 2, PrgBank::WORK_RAM.fixed_index(0).status_register(S0)),
        PrgWindow::new(0x7A00, 0x7BFF, KIBIBYTE / 2, PrgBank::WORK_RAM.fixed_index(1).status_register(S1)),
        PrgWindow::new(0x7C00, 0x7DFF, KIBIBYTE / 2, PrgBank::WORK_RAM.fixed_index(0).status_register(S0)),
        PrgWindow::new(0x7E00, 0x7FFF, KIBIBYTE / 2, PrgBank::WORK_RAM.fixed_index(1).status_register(S1)),
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
    .read_write_statuses(&[
        ReadWriteStatus::ReadOnly,
        ReadWriteStatus::ReadWrite,
    ])
    .build();

// MMC6. Similar to MMC3 with Sharp IRQs, but with Work RAM protection.
// TODO: Support VS System (and its 4-screen mirroring).
pub struct Mapper004_1 {
    selected_register_id: RegId,
    irq_state: RevAIrqState,
}

impl Mapper for Mapper004_1 {
    fn write_register(&mut self, params: &mut MapperParams, cpu_address: u16, value: u8) {
        let is_even_address = cpu_address.is_multiple_of(2);
        match (cpu_address, is_even_address) {
            (0x0000..=0x401F, _) => unreachable!(),
            (0x4020..=0x7FFF, _) => { /* Do nothing. */ }
            (0x8000..=0x9FFF, true ) => self.bank_select(params, value),
            (0x8000..=0x9FFF, false) => mmc3::set_bank_index(params, &mut self.selected_register_id, value),
            (0xA000..=0xBFFF, true ) => Self::set_mirroring(params, value),
            (0xA000..=0xBFFF, false) => Self::prg_ram_protect(params, value),
            (0xC000..=0xDFFF, true ) => self.irq_state.set_counter_reload_value(value),
            (0xC000..=0xDFFF, false) => self.irq_state.reload_counter(),
            (0xE000..=0xFFFF, true ) => self.irq_state.disable(params),
            (0xE000..=0xFFFF, false) => self.irq_state.enable(),
        }
    }

    fn on_end_of_ppu_cycle(&mut self) {
        self.irq_state.decrement_suppression_cycle_count();
    }

    fn on_ppu_address_change(&mut self, params: &mut MapperParams, address: PpuAddress) {
        self.irq_state.tick_counter(params, address);
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}

impl Mapper004_1 {
    // Same as MMC3 except for PRG RAM enable and slightly different PRG layouts.
    pub fn bank_select(&mut self, params: &mut MapperParams, value: u8) {
        let fields = splitbits!(min=u8, value, "cps..bbb");
        params.set_chr_layout(fields.c);
        params.set_prg_layout(fields.p);
        // FIXME: What are these actually supposed to do?
        params.set_read_write_status(S0, fields.s);
        params.set_read_write_status(S1, fields.s);
        self.selected_register_id = mmc3::BANK_INDEX_REGISTER_IDS[fields.b as usize];
    }

    pub fn set_mirroring(params: &mut MapperParams, value: u8) {
        // Hard-coded 4-screen mirroring cannot be overridden.
        if params.name_table_mirroring().is_vertical() || params.name_table_mirroring().is_horizontal() {
            params.set_name_table_mirroring(value & 1);
        }
    }

    // TODO: This should be implementable now.
    pub fn prg_ram_protect(_params: &mut MapperParams, _value: u8) {
        // TODO: Once NES 2.0 is supported, then MMC3 and MMC6 can properly be supported.
        /*
        if !self.prg_ram_enabled {
            return;
        }

        // MMC6 logic only here since MMC3 logic conflicts:
        // https://www.nesdev.org/wiki/MMC3#iNES_Mapper_004_and_MMC6
        // TODO: Attempt to support Low G Man.
        let mut status_7000 = Mapper004::work_ram_status_from_bits(value & 0b1100_0000 >> 6);
        let mut status_7200 = Mapper004::work_ram_status_from_bits(value & 0b0011_0000 >> 4);

        // "If only one bank is enabled for reading, the other reads back as zero."
        use WorkRamStatus::*;
        match (status_7000, status_7200) {
            (ReadOnly | ReadWrite, Disabled            ) => status_7200 = ReadOnlyZeros,
            (Disabled            , ReadOnly | ReadWrite) => status_7000 = ReadOnlyZeros,
        }

        self.prg_memory.set_work_ram_status_at(0x7000, status_7000);
        self.prg_memory.set_work_ram_status_at(0x7200, status_7200);
        */
    }

    pub fn new() -> Self {
        Self {
            selected_register_id: RegId::Chr(C0),
            irq_state: RevAIrqState::new(),
        }
    }
}

/*
fn work_ram_status_from_bits(value: u8) -> WorkRamStatus {
    assert_eq!(value & 0b1111_1100, 0);

    match value {
        0b00 => WorkRamStatus::Disabled,
        0b01 => WorkRamStatus::Disabled,
        0b10 => WorkRamStatus::ReadOnly,
        0b11 => WorkRamStatus::ReadWrite,
        _ => unreachable!(),
    }
}
*/
