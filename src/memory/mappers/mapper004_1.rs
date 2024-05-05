use crate::memory::mapper::*;
use crate::memory::mappers::mmc3::mmc3;
use crate::memory::mappers::mmc3::irq_state::IrqState;
use crate::memory::mappers::mmc3::rev_a_irq_state::RevAIrqState;

const PRG_LAYOUT_8000_SWITCHABLE: PrgLayout = PrgLayout::new(&[
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
]);

const PRG_LAYOUT_C000_SWITCHABLE: PrgLayout = PrgLayout::new(&[
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
]);

pub const INITIAL_LAYOUT: InitialLayout = InitialLayout::builder()
    .prg_max_bank_count(64)
    .prg_bank_size(8 * KIBIBYTE)
    .prg_windows(PRG_LAYOUT_8000_SWITCHABLE)
    .chr_max_bank_count(256)
    .chr_bank_size(1 * KIBIBYTE)
    .chr_windows(mmc3::CHR_BIG_WINDOWS_FIRST)
    .name_table_mirroring_source(NameTableMirroringSource::Cartridge)
    .build();


// MMC6. Similar to MMC3 with Sharp IRQs, but with Work RAM protection.
pub struct Mapper004_1 {
    selected_register_id: BankRegisterId,
    irq_state: RevAIrqState,
    prg_ram_enabled: bool,
}

impl Mapper for Mapper004_1 {
    fn initial_layout(&self) -> InitialLayout {
        INITIAL_LAYOUT
    }

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
}

impl Mapper004_1 {
    // Same as MMC3 except for PRG RAM enable and slightly different PRG layouts.
    pub fn bank_select(&mut self, params: &mut MapperParams, value: u8) {
        let chr_big_windows_first =                               (value & 0b1000_0000) == 0;
        let prg_switchable_8000 =                                 (value & 0b0100_0000) == 0;
        self.prg_ram_enabled =                                    (value & 0b0010_0000) != 0;
        self.selected_register_id = mmc3::BANK_INDEX_REGISTER_IDS[(value & 0b0000_0111) as usize];

        if chr_big_windows_first {
            params.set_chr_layout(mmc3::CHR_BIG_WINDOWS_FIRST);
        } else {
            params.set_chr_layout(mmc3::CHR_SMALL_WINDOWS_FIRST);
        }

        if prg_switchable_8000 {
            params.set_prg_layout(PRG_LAYOUT_8000_SWITCHABLE);
        } else {
            params.set_prg_layout(PRG_LAYOUT_C000_SWITCHABLE);
        }
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
            prg_ram_enabled: false,
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
