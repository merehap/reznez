use crate::memory::mapper::*;
use crate::memory::mappers::mmc3::irq_state::IrqState;

pub const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(512 * KIBIBYTE)
    .prg_layout(PRG_WINDOWS_8000_SWITCHABLE)
    .prg_layout(PRG_WINDOWS_C000_SWITCHABLE)
    .chr_rom_max_size(256 * KIBIBYTE)
    .chr_layout(CHR_BIG_WINDOWS_FIRST)
    .chr_layout(CHR_SMALL_WINDOWS_FIRST)
    .name_table_mirrorings(&[
        NameTableMirroring::VERTICAL,
        NameTableMirroring::HORIZONTAL,
    ])
    .ram_statuses(&[
        RamStatus::Disabled,
        RamStatus::ReadOnly,
        RamStatus::ReadWrite,
        RamStatus::ReadOnly,
    ])
    .build();

pub const PRG_WINDOWS_8000_SWITCHABLE: &[Window] = &[
    Window::new(0x6000, 0x7FFF, 8 * KIBIBYTE, Bank::WORK_RAM.status_register(S0)),
    Window::new(0x8000, 0x9FFF, 8 * KIBIBYTE, Bank::ROM.switchable(P0)),
    Window::new(0xA000, 0xBFFF, 8 * KIBIBYTE, Bank::ROM.switchable(P1)),
    Window::new(0xC000, 0xDFFF, 8 * KIBIBYTE, Bank::ROM.fixed_index(-2)),
    Window::new(0xE000, 0xFFFF, 8 * KIBIBYTE, Bank::ROM.fixed_index(-1)),
];

pub const PRG_WINDOWS_C000_SWITCHABLE: &[Window] = &[
    Window::new(0x6000, 0x7FFF, 8 * KIBIBYTE, Bank::WORK_RAM.status_register(S0)),
    Window::new(0x8000, 0x9FFF, 8 * KIBIBYTE, Bank::ROM.fixed_index(-2)),
    Window::new(0xA000, 0xBFFF, 8 * KIBIBYTE, Bank::ROM.switchable(P1)),
    Window::new(0xC000, 0xDFFF, 8 * KIBIBYTE, Bank::ROM.switchable(P0)),
    Window::new(0xE000, 0xFFFF, 8 * KIBIBYTE, Bank::ROM.fixed_index(-1)),
];

pub const CHR_BIG_WINDOWS_FIRST: &[Window] = &[
    // Big windows.
    Window::new(0x0000, 0x07FF, 2 * KIBIBYTE, Bank::ROM.switchable(C0)),
    Window::new(0x0800, 0x0FFF, 2 * KIBIBYTE, Bank::ROM.switchable(C1)),
    // Small windows.
    Window::new(0x1000, 0x13FF, 1 * KIBIBYTE, Bank::ROM.switchable(C2)),
    Window::new(0x1400, 0x17FF, 1 * KIBIBYTE, Bank::ROM.switchable(C3)),
    Window::new(0x1800, 0x1BFF, 1 * KIBIBYTE, Bank::ROM.switchable(C4)),
    Window::new(0x1C00, 0x1FFF, 1 * KIBIBYTE, Bank::ROM.switchable(C5)),
];

pub const CHR_SMALL_WINDOWS_FIRST: &[Window] = &[
    // Small windows.
    Window::new(0x0000, 0x03FF, 1 * KIBIBYTE, Bank::ROM.switchable(C2)),
    Window::new(0x0400, 0x07FF, 1 * KIBIBYTE, Bank::ROM.switchable(C3)),
    Window::new(0x0800, 0x0BFF, 1 * KIBIBYTE, Bank::ROM.switchable(C4)),
    Window::new(0x0C00, 0x0FFF, 1 * KIBIBYTE, Bank::ROM.switchable(C5)),
    // Big windows.
    Window::new(0x1000, 0x17FF, 2 * KIBIBYTE, Bank::ROM.switchable(C0)),
    Window::new(0x1800, 0x1FFF, 2 * KIBIBYTE, Bank::ROM.switchable(C1)),
];


pub const BANK_INDEX_REGISTER_IDS: [BankRegisterId; 8] = [C0, C1, C2, C3, C4, C5, P0, P1];

pub struct Mapper004Mmc3 {
    selected_register_id: BankRegisterId,
    irq_state: Box<dyn IrqState>,
}

impl Mapper for Mapper004Mmc3 {
    fn write_to_cartridge_space(&mut self, params: &mut MapperParams, cpu_address: u16, value: u8) {
        let is_even_address = cpu_address % 2 == 0;
        match (cpu_address, is_even_address) {
            (0x0000..=0x401F, _) => unreachable!(),
            (0x4020..=0x7FFF, _) => { /* Do nothing. */ }
            (0x8000..=0x9FFF, true ) => bank_select(params, &mut self.selected_register_id, value),
            (0x8000..=0x9FFF, false) => set_bank_index(params, &mut self.selected_register_id, value),
            (0xA000..=0xBFFF, true ) => set_mirroring(params, value),
            (0xA000..=0xBFFF, false) => prg_ram_protect(params, value),
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
    let fields = splitbits!(min=u8, value, "cp...rrr");
    params.set_chr_layout(fields.c);
    params.set_prg_layout(fields.p);
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
    // Cartridge hard-coded 4-screen mirroring cannot be changed.
    if params.name_table_mirroring().is_vertical() || params.name_table_mirroring().is_horizontal() {
        params.set_name_table_mirroring(value & 1);
    }
}

pub fn prg_ram_protect(params: &mut MapperParams, value: u8) {
    params.set_ram_status(S0, value >> 6);
}
