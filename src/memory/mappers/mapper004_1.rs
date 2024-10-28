use crate::memory::mapper::*;
use crate::memory::mappers::mmc3::mmc3;
use crate::memory::mappers::mmc3::irq_state::IrqState;
use crate::memory::mappers::mmc3::rev_a_irq_state::RevAIrqState;

const LAYOUT: Layout = Layout::builder()
    .prg_max_size(512 * KIBIBYTE)
    .chr_max_size(256 * KIBIBYTE)
    // Switchable 0x8000
    .prg_layout(PrgLayout::new(&[
        PrgWindow::new(0x6000, 0x6FFF, 4 * KIBIBYTE, Bank::EMPTY),
        PrgWindow::new(0x7000, 0x71FF, KIBIBYTE / 2, Bank::WORK_RAM.status_register(S0)),
        PrgWindow::new(0x7200, 0x73FF, KIBIBYTE / 2, Bank::WORK_RAM.status_register(S1)),
        PrgWindow::new(0x7400, 0x75FF, KIBIBYTE / 2, Bank::MirrorOf(0x7000)),
        PrgWindow::new(0x7600, 0x77FF, KIBIBYTE / 2, Bank::MirrorOf(0x7200)),
        PrgWindow::new(0x7800, 0x79FF, KIBIBYTE / 2, Bank::MirrorOf(0x7000)),
        PrgWindow::new(0x7A00, 0x7BFF, KIBIBYTE / 2, Bank::MirrorOf(0x7200)),
        PrgWindow::new(0x7C00, 0x7DFF, KIBIBYTE / 2, Bank::MirrorOf(0x7000)),
        PrgWindow::new(0x7E00, 0x7FFF, KIBIBYTE / 2, Bank::MirrorOf(0x7200)),
        PrgWindow::new(0x8000, 0x9FFF, 8 * KIBIBYTE, Bank::switchable_rom(P0)),
        PrgWindow::new(0xA000, 0xBFFF, 8 * KIBIBYTE, Bank::switchable_rom(P1)),
        PrgWindow::new(0xC000, 0xDFFF, 8 * KIBIBYTE, Bank::fixed_rom(BankIndex::SECOND_LAST)),
        PrgWindow::new(0xE000, 0xFFFF, 8 * KIBIBYTE, Bank::fixed_rom(BankIndex::LAST)),
    ]))
    // Switchable 0xC000
    .prg_layout(PrgLayout::new(&[
        PrgWindow::new(0x6000, 0x6FFF, 4 * KIBIBYTE, Bank::EMPTY),
        PrgWindow::new(0x7000, 0x71FF, KIBIBYTE / 2, Bank::WORK_RAM.status_register(S0)),
        PrgWindow::new(0x7200, 0x73FF, KIBIBYTE / 2, Bank::WORK_RAM.status_register(S1)),
        PrgWindow::new(0x7400, 0x75FF, KIBIBYTE / 2, Bank::MirrorOf(0x7000)),
        PrgWindow::new(0x7600, 0x77FF, KIBIBYTE / 2, Bank::MirrorOf(0x7200)),
        PrgWindow::new(0x7800, 0x79FF, KIBIBYTE / 2, Bank::MirrorOf(0x7000)),
        PrgWindow::new(0x7A00, 0x7BFF, KIBIBYTE / 2, Bank::MirrorOf(0x7200)),
        PrgWindow::new(0x7C00, 0x7DFF, KIBIBYTE / 2, Bank::MirrorOf(0x7000)),
        PrgWindow::new(0x7E00, 0x7FFF, KIBIBYTE / 2, Bank::MirrorOf(0x7200)),
        PrgWindow::new(0x8000, 0x9FFF, 8 * KIBIBYTE, Bank::fixed_rom(BankIndex::SECOND_LAST)),
        PrgWindow::new(0xA000, 0xBFFF, 8 * KIBIBYTE, Bank::switchable_rom(P1)),
        PrgWindow::new(0xC000, 0xDFFF, 8 * KIBIBYTE, Bank::switchable_rom(P0)),
        PrgWindow::new(0xE000, 0xFFFF, 8 * KIBIBYTE, Bank::fixed_rom(BankIndex::LAST)),
    ]))
    .chr_layout(mmc3::CHR_BIG_WINDOWS_FIRST)
    .chr_layout(mmc3::CHR_SMALL_WINDOWS_FIRST)
    .build();

const RAM_STATUSES: [RamStatus; 2] =
    [
        RamStatus::ReadOnly,
        RamStatus::ReadWrite,
    ];

// MMC6. Similar to MMC3 with Sharp IRQs, but with Work RAM protection.
pub struct Mapper004_1 {
    selected_register_id: BankRegisterId,
    irq_state: RevAIrqState,
}

impl Mapper for Mapper004_1 {
    fn write_to_cartridge_space(&mut self, params: &mut MapperParams, address: CpuAddress, value: u8) {
        let is_even_address = address.to_raw() % 2 == 0;
        match (address.to_raw(), is_even_address) {
            (0x0000..=0x401F, _) => unreachable!(),
            (0x4020..=0x5FFF, _) => { /* Do nothing. */ }
            (0x6000..=0x7FFF, _) => params.write_prg(address, value),
            (0x8000..=0x9FFF, true ) => self.bank_select(params, value),
            (0x8000..=0x9FFF, false) => mmc3::set_bank_index(params, &mut self.selected_register_id, value),
            (0xA000..=0xBFFF, true ) => mmc3::set_mirroring(params, value),
            (0xA000..=0xBFFF, false) => Mapper004_1::prg_ram_protect(params, value),
            (0xC000..=0xDFFF, true ) => self.irq_state.set_counter_reload_value(value),
            (0xC000..=0xDFFF, false) => self.irq_state.reload_counter(),
            (0xE000..=0xFFFF, true ) => self.irq_state.disable(),
            (0xE000..=0xFFFF, false) => self.irq_state.enable(),
        }
    }

    fn on_end_of_ppu_cycle(&mut self) {
        self.irq_state.decrement_suppression_cycle_count();
    }

    fn process_current_ppu_address(&mut self, address: PpuAddress) {
        self.irq_state.tick_counter(address);
    }

    fn irq_pending(&self) -> bool {
        self.irq_state.pending()
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}

impl Mapper004_1 {
    // Same as MMC3 except for PRG RAM enable and slightly different PRG layouts.
    pub fn bank_select(&mut self, params: &mut MapperParams, value: u8) {
        let fields = splitbits!(value, "cps..bbb");
        params.set_chr_layout(fields.c as usize);
        params.set_prg_layout(fields.p as usize);
        // FIXME: What are these actually supposed to do?
        params.set_ram_status(S0, RAM_STATUSES[fields.s as usize]);
        params.set_ram_status(S1, RAM_STATUSES[fields.s as usize]);
        self.selected_register_id = mmc3::BANK_INDEX_REGISTER_IDS[fields.b as usize];
    }

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
            selected_register_id: C0,
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
