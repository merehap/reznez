use crate::memory::mapper::*;
use crate::memory::mappers::mmc3::irq_state::IrqState;

pub const PRG_LAYOUT_8000_SWITCHABLE: PrgLayout = PrgLayout::new(&[
    PrgWindow::new(0x6000, 0x7FFF, 8 * KIBIBYTE, Bank::WORK_RAM.status_register(S0)),
    PrgWindow::new(0x8000, 0x9FFF, 8 * KIBIBYTE, Bank::switchable_rom(P0)),
    PrgWindow::new(0xA000, 0xBFFF, 8 * KIBIBYTE, Bank::switchable_rom(P1)),
    PrgWindow::new(0xC000, 0xDFFF, 8 * KIBIBYTE, Bank::fixed_rom(BankIndex::SECOND_LAST)),
    PrgWindow::new(0xE000, 0xFFFF, 8 * KIBIBYTE, Bank::fixed_rom(BankIndex::LAST)),
]);

pub const PRG_LAYOUT_C000_SWITCHABLE: PrgLayout = PrgLayout::new(&[
    PrgWindow::new(0x6000, 0x7FFF, 8 * KIBIBYTE, Bank::WORK_RAM.status_register(S0)),
    PrgWindow::new(0x8000, 0x9FFF, 8 * KIBIBYTE, Bank::fixed_rom(BankIndex::SECOND_LAST)),
    PrgWindow::new(0xA000, 0xBFFF, 8 * KIBIBYTE, Bank::switchable_rom(P1)),
    PrgWindow::new(0xC000, 0xDFFF, 8 * KIBIBYTE, Bank::switchable_rom(P0)),
    PrgWindow::new(0xE000, 0xFFFF, 8 * KIBIBYTE, Bank::fixed_rom(BankIndex::LAST)),
]);

pub const CHR_BIG_WINDOWS_FIRST: ChrLayout = ChrLayout::new(&[
    // Big windows.
    ChrWindow::new(0x0000, 0x07FF, 2 * KIBIBYTE, Bank::switchable_rom(C0)),
    ChrWindow::new(0x0800, 0x0FFF, 2 * KIBIBYTE, Bank::switchable_rom(C1)),
    // Small windows.
    ChrWindow::new(0x1000, 0x13FF, 1 * KIBIBYTE, Bank::switchable_rom(C2)),
    ChrWindow::new(0x1400, 0x17FF, 1 * KIBIBYTE, Bank::switchable_rom(C3)),
    ChrWindow::new(0x1800, 0x1BFF, 1 * KIBIBYTE, Bank::switchable_rom(C4)),
    ChrWindow::new(0x1C00, 0x1FFF, 1 * KIBIBYTE, Bank::switchable_rom(C5)),
]);

pub const CHR_SMALL_WINDOWS_FIRST: ChrLayout = ChrLayout::new(&[
    // Small windows.
    ChrWindow::new(0x0000, 0x03FF, 1 * KIBIBYTE, Bank::switchable_rom(C2)),
    ChrWindow::new(0x0400, 0x07FF, 1 * KIBIBYTE, Bank::switchable_rom(C3)),
    ChrWindow::new(0x0800, 0x0BFF, 1 * KIBIBYTE, Bank::switchable_rom(C4)),
    ChrWindow::new(0x0C00, 0x0FFF, 1 * KIBIBYTE, Bank::switchable_rom(C5)),
    // Big windows.
    ChrWindow::new(0x1000, 0x17FF, 2 * KIBIBYTE, Bank::switchable_rom(C0)),
    ChrWindow::new(0x1800, 0x1FFF, 2 * KIBIBYTE, Bank::switchable_rom(C1)),
]);

const CHR_LAYOUTS: [ChrLayout; 2] = [CHR_BIG_WINDOWS_FIRST, CHR_SMALL_WINDOWS_FIRST];
const PRG_LAYOUTS: [PrgLayout; 2] = [PRG_LAYOUT_8000_SWITCHABLE, PRG_LAYOUT_C000_SWITCHABLE];

pub const INITIAL_LAYOUT: InitialLayout = InitialLayout::builder()
    .prg_max_bank_count(64)
    .prg_bank_size(8 * KIBIBYTE)
    .prg_windows(PRG_LAYOUT_8000_SWITCHABLE)
    .chr_max_bank_count(256)
    .chr_bank_size(1 * KIBIBYTE)
    .chr_windows(CHR_BIG_WINDOWS_FIRST)
    .name_table_mirroring_source(NameTableMirroringSource::Cartridge)
    .build();

pub const BANK_INDEX_REGISTER_IDS: [BankRegisterId; 8] = [C0, C1, C2, C3, C4, C5, P0, P1];

pub struct Mapper004Mmc3 {
    selected_register_id: BankRegisterId,
    irq_state: Box<dyn IrqState>,
}

impl Mapper for Mapper004Mmc3 {
    fn initial_layout(&self) -> InitialLayout {
        INITIAL_LAYOUT
    }

    fn write_to_cartridge_space(&mut self, params: &mut MapperParams, address: CpuAddress, value: u8) {
        let is_even_address = address.to_raw() % 2 == 0;
        match (address.to_raw(), is_even_address) {
            (0x0000..=0x401F, _) => unreachable!(),
            (0x4020..=0x5FFF, _) => { /* Do nothing. */ }
            (0x6000..=0x7FFF, _) => params.write_prg(address, value),
            (0x8000..=0x9FFF, true ) => bank_select(params, &mut self.selected_register_id, value),
            (0x8000..=0x9FFF, false) => set_bank_index(params, &mut self.selected_register_id, value),
            (0xA000..=0xBFFF, true ) => set_mirroring(params, value),
            (0xA000..=0xBFFF, false) => prg_ram_protect(params, value),
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

impl Mapper004Mmc3 {
    pub fn new(irq_state: Box<dyn IrqState>) -> Self {
        Self {
            selected_register_id: C0,
            irq_state,
        }
    }
}

pub fn bank_select(
    params: &mut MapperParams,
    selected_register_id: &mut BankRegisterId,
    value: u8,
) {
    let fields = splitbits!(value, "cp...rrr");
    params.set_chr_layout(CHR_LAYOUTS[fields.c as usize]);
    params.set_prg_layout(PRG_LAYOUTS[fields.p as usize]);
    *selected_register_id = BANK_INDEX_REGISTER_IDS[fields.r as usize];
}

pub fn set_bank_index(
    params: &mut MapperParams,
    selected_register_id: &mut BankRegisterId,
    value: u8,
) {
    let mut bank_index = value;
    if matches!(*selected_register_id, C0 | C1) {
        // Double-width windows can only use even banks.
        bank_index &= 0b1111_1110;
    }

    if matches!(*selected_register_id, P0 | P1) {
        // "Some romhacks rely on an 8-bit extension of R6/7 for oversized PRG-ROM,
        // but this is deliberately not supported by many emulators."
        bank_index &= 0b0011_1111;
    }

    params.set_bank_register(*selected_register_id, bank_index);
}

pub fn set_mirroring(params: &mut MapperParams, value: u8) {
    // TODO: splitbits single 
    use NameTableMirroring::*;
    match (params.name_table_mirroring(), value & 0b0000_0001) {
        (Vertical, 1) => params.set_name_table_mirroring(Horizontal),
        (Horizontal, 0) => params.set_name_table_mirroring(Vertical),
        _ => { /* Other mirrorings cannot be changed. */ }
    }
}

pub fn prg_ram_protect(params: &mut MapperParams, value: u8) {
    // TODO: splitbits tuple
    let read_only  = value & 0b0100_0000 != 0;
    let enable_ram = value & 0b1000_0000 != 0;

    let status = if read_only {
        RamStatus::ReadOnly
    } else if enable_ram {
        RamStatus::ReadWrite
    } else {
        RamStatus::Disabled
    };
    params.set_ram_status(S0, status);
}
