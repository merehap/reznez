use crate::memory::mapper::*;

const PRG_LAYOUT_PRIMARY: PrgLayout = PrgLayout::new(&[
    PrgWindow::new(0x6000, 0x7FFF, 8 * KIBIBYTE, Bank::EMPTY),
    PrgWindow::new(0x8000, 0x9FFF, 8 * KIBIBYTE, Bank::switchable_rom(P0)),
    PrgWindow::new(0xA000, 0xBFFF, 8 * KIBIBYTE, Bank::switchable_rom(P1)),
    PrgWindow::new(0xC000, 0xDFFF, 8 * KIBIBYTE, Bank::switchable_rom(P2)),
    PrgWindow::new(0xE000, 0xFFFF, 8 * KIBIBYTE, Bank::fixed_rom(BankIndex::LAST)),
]);

const PRG_LAYOUT_SECONDARY: PrgLayout = PrgLayout::new(&[
    PrgWindow::new(0x6000, 0x7FFF, 8 * KIBIBYTE, Bank::EMPTY),
    PrgWindow::new(0x8000, 0x9FFF, 8 * KIBIBYTE, Bank::switchable_rom(P2)),
    PrgWindow::new(0xA000, 0xBFFF, 8 * KIBIBYTE, Bank::switchable_rom(P1)),
    PrgWindow::new(0xC000, 0xDFFF, 8 * KIBIBYTE, Bank::switchable_rom(P0)),
    PrgWindow::new(0xE000, 0xFFFF, 8 * KIBIBYTE, Bank::fixed_rom(BankIndex::LAST)),
]);

const CHR_BIG_WINDOWS_PRIMARY: ChrLayout = ChrLayout::new(&[
    // Big windows.
    ChrWindow::new(0x0000, 0x07FF, 2 * KIBIBYTE, Bank::switchable_rom(C0)),
    ChrWindow::new(0x0800, 0x0FFF, 2 * KIBIBYTE, Bank::switchable_rom(C1)),
    // Small windows.
    ChrWindow::new(0x1000, 0x13FF, 1 * KIBIBYTE, Bank::switchable_rom(C2)),
    ChrWindow::new(0x1400, 0x17FF, 1 * KIBIBYTE, Bank::switchable_rom(C3)),
    ChrWindow::new(0x1800, 0x1BFF, 1 * KIBIBYTE, Bank::switchable_rom(C4)),
    ChrWindow::new(0x1C00, 0x1FFF, 1 * KIBIBYTE, Bank::switchable_rom(C5)),
]);

const CHR_BIG_WINDOWS_SECONDARY: ChrLayout = ChrLayout::new(&[
    // Small windows.
    ChrWindow::new(0x0000, 0x03FF, 1 * KIBIBYTE, Bank::switchable_rom(C2)),
    ChrWindow::new(0x0400, 0x07FF, 1 * KIBIBYTE, Bank::switchable_rom(C3)),
    ChrWindow::new(0x0800, 0x0BFF, 1 * KIBIBYTE, Bank::switchable_rom(C4)),
    ChrWindow::new(0x0C00, 0x0FFF, 1 * KIBIBYTE, Bank::switchable_rom(C5)),
    // Big windows.
    ChrWindow::new(0x1000, 0x17FF, 2 * KIBIBYTE, Bank::switchable_rom(C0)),
    ChrWindow::new(0x1800, 0x1FFF, 2 * KIBIBYTE, Bank::switchable_rom(C1)),
]);

const CHR_SMALL_WINDOWS_PRIMARY: ChrLayout = ChrLayout::new(&[
    ChrWindow::new(0x0000, 0x03FF, 1 * KIBIBYTE, Bank::switchable_rom(C0)),
    ChrWindow::new(0x0400, 0x07FF, 1 * KIBIBYTE, Bank::switchable_rom(C6)),
    ChrWindow::new(0x0800, 0x0BFF, 1 * KIBIBYTE, Bank::switchable_rom(C1)),
    ChrWindow::new(0x0C00, 0x0FFF, 1 * KIBIBYTE, Bank::switchable_rom(C7)),
    ChrWindow::new(0x1000, 0x13FF, 1 * KIBIBYTE, Bank::switchable_rom(C2)),
    ChrWindow::new(0x1400, 0x17FF, 1 * KIBIBYTE, Bank::switchable_rom(C3)),
    ChrWindow::new(0x1800, 0x1BFF, 1 * KIBIBYTE, Bank::switchable_rom(C4)),
    ChrWindow::new(0x1C00, 0x1FFF, 1 * KIBIBYTE, Bank::switchable_rom(C5)),
]);

const CHR_SMALL_WINDOWS_SECONDARY: ChrLayout = ChrLayout::new(&[
    ChrWindow::new(0x0000, 0x03FF, 1 * KIBIBYTE, Bank::switchable_rom(C2)),
    ChrWindow::new(0x0400, 0x07FF, 1 * KIBIBYTE, Bank::switchable_rom(C3)),
    ChrWindow::new(0x0800, 0x0BFF, 1 * KIBIBYTE, Bank::switchable_rom(C4)),
    ChrWindow::new(0x0C00, 0x0FFF, 1 * KIBIBYTE, Bank::switchable_rom(C5)),
    ChrWindow::new(0x1000, 0x13FF, 1 * KIBIBYTE, Bank::switchable_rom(C0)),
    ChrWindow::new(0x1400, 0x17FF, 1 * KIBIBYTE, Bank::switchable_rom(C6)),
    ChrWindow::new(0x1800, 0x1BFF, 1 * KIBIBYTE, Bank::switchable_rom(C1)),
    ChrWindow::new(0x1C00, 0x1FFF, 1 * KIBIBYTE, Bank::switchable_rom(C7)),
]);

const CPU_CYCLE_MODE_IRQ_PENDING_DELAY: u8 = 1;
const SCANLINE_MODE_IRQ_PENDING_DELAY: u8 = 4;

const BANK_INDEX_REGISTER_IDS: [Option<BankRegisterId>; 16] =
    [Some(C0), Some(C1), Some(C2), Some(C3), Some(C4), Some(C5), Some(P0), Some(P1),
     Some(C6), Some(C7),     None,     None,     None,     None,     None, Some(P2),
    ];

const PRG_LAYOUTS: [PrgLayout; 2] = [
    PRG_LAYOUT_PRIMARY,
    PRG_LAYOUT_SECONDARY,
];

const CHR_LAYOUTS: [[ChrLayout; 2]; 2] = [
    [ CHR_BIG_WINDOWS_PRIMARY, CHR_SMALL_WINDOWS_PRIMARY ],
    [ CHR_BIG_WINDOWS_SECONDARY, CHR_SMALL_WINDOWS_SECONDARY ],
];

const MIRRORINGS: [NameTableMirroring; 2] = [
    NameTableMirroring::Vertical,
    NameTableMirroring::Horizontal,
];

// RAMBO-1 (Similar to MMC3)
pub struct Mapper064 {
    selected_register_id: BankRegisterId,

    irq_enabled: bool,
    irq_pending: bool,
    irq_pending_delay_cycles: u8,

    irq_counter: u8,
    force_reload_irq_counter: bool,
    irq_counter_reload_value: u8,
    irq_counter_reload_mode: IrqCounterReloadMode,
    irq_counter_suppression_cycles: u8,
    pattern_table_side: PatternTableSide,
}

impl Mapper for Mapper064 {
    fn initial_layout(&self) -> InitialLayout {
        InitialLayout::builder()
            .prg_max_bank_count(64)
            .prg_bank_size(8 * KIBIBYTE)
            .prg_windows(PRG_LAYOUT_PRIMARY)
            .chr_max_bank_count(256)
            .chr_bank_size(1 * KIBIBYTE)
            .chr_windows(CHR_BIG_WINDOWS_PRIMARY)
            .name_table_mirroring_source(NameTableMirroringSource::Cartridge)
            .build()
    }

    fn write_to_cartridge_space(&mut self, params: &mut MapperParams, cpu_address: CpuAddress, value: u8) {
        let is_even_address = cpu_address.to_raw() % 2 == 0;
        match cpu_address.to_raw() {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x7FFF => { /* Do nothing. */ }
            0x8000..=0x9FFF if is_even_address => self.bank_select(params, value),
            0x8000..=0x9FFF => self.set_bank_index(params, value),
            0xA000..=0xBFFF if is_even_address => Mapper064::set_name_table_mirroring(params, value),
            0xA000..=0xBFFF => {/* Do nothing. No use of these registers has been found. */}
            0xC000..=0xDFFF if is_even_address => self.set_irq_counter_reload_value(value), 
            0xC000..=0xDFFF => self.set_irq_reload_mode(value),
            0xE000..=0xFFFF if is_even_address => self.acknowledge_irq(),
            0xE000..=0xFFFF => self.enabled_irq()
        }
    }

    fn on_end_of_cpu_cycle(&mut self, cycle: i64) {
        if self.irq_pending_delay_cycles > 0 {
            self.irq_pending_delay_cycles -= 1;
            if self.irq_pending_delay_cycles == 0 {
                self.irq_pending = true;
            }
        }

        if self.irq_counter_reload_mode == IrqCounterReloadMode::CpuCycle && cycle % 4 == 0 {
            self.tick_irq_counter(CPU_CYCLE_MODE_IRQ_PENDING_DELAY);
        }
    }


    fn on_end_of_ppu_cycle(&mut self) {
        if self.irq_counter_suppression_cycles > 0 {
            self.irq_counter_suppression_cycles -= 1;
        }
    }

    // When in scanline reload mode, this is the same as MMC3's IRQ triggering except delayed a bit.
    fn process_current_ppu_address(&mut self, address: PpuAddress) {
        if self.irq_counter_reload_mode != IrqCounterReloadMode::Scanline {
            return;
        }

        // Only process pattern table fetches.
        if !address.is_in_pattern_table() {
            return;
        }

        let next_side = address.pattern_table_side();
        let should_tick_irq_counter =
            self.pattern_table_side == PatternTableSide::Left
            && next_side == PatternTableSide::Right
            && self.irq_counter_suppression_cycles == 0;
        if next_side == PatternTableSide::Right {
            self.irq_counter_suppression_cycles = 16;
        }

        if should_tick_irq_counter {
            self.tick_irq_counter(SCANLINE_MODE_IRQ_PENDING_DELAY);
        }

        self.pattern_table_side = next_side;
    }

    fn irq_pending(&self) -> bool {
        self.irq_pending
    }
}

impl Mapper064 {
    pub fn new() -> Self {
        Self {
            selected_register_id: C0,

            irq_enabled: false,
            irq_pending: false,
            irq_pending_delay_cycles: 0,

            irq_counter: 0,
            force_reload_irq_counter: false,
            irq_counter_reload_value: 0,
            irq_counter_reload_mode: IrqCounterReloadMode::Scanline,
            irq_counter_suppression_cycles: 0,
            pattern_table_side: PatternTableSide::Left,
        }
    }

    fn bank_select(&mut self, params: &mut MapperParams, value: u8) {
        let fields = splitbits!(value, "ipc.bbbb");
        params.set_prg_layout(PRG_LAYOUTS[fields.p as usize]);
        params.set_chr_layout(CHR_LAYOUTS[fields.i as usize][fields.c as usize]);
        if let Some(reg_id) = BANK_INDEX_REGISTER_IDS[fields.b as usize] {
            self.selected_register_id = reg_id;
        }
    }

    fn set_bank_index(&self, params: &mut MapperParams, value: u8) {
        params.set_bank_register(self.selected_register_id, value);
    }

    fn set_name_table_mirroring(params: &mut MapperParams, value: u8) {
        params.set_name_table_mirroring(MIRRORINGS[usize::from(value & 1)]);
    }

    fn set_irq_counter_reload_value(&mut self, value: u8) {
        self.irq_counter_reload_value = value;
    }

    fn set_irq_reload_mode(&mut self, value: u8) {
        self.force_reload_irq_counter = true;
        // TODO: reset the prescaler in cycle mode, so the next clock will occur 4 cycles later.
        self.irq_counter_reload_mode = if value & 1 == 0 {
            IrqCounterReloadMode::Scanline
        } else {
            IrqCounterReloadMode::CpuCycle
        };
    }

    fn acknowledge_irq(&mut self) {
        self.irq_enabled = false;
        self.irq_pending = false;
    }

    fn enabled_irq(&mut self) {
        self.irq_enabled = true;
    }

    fn tick_irq_counter(&mut self, irq_pending_delay_cycles: u8) {
        if self.force_reload_irq_counter {
            self.force_reload_irq_counter = false;
            self.irq_counter = self.irq_counter_reload_value;
            if self.irq_counter > 0 {
                self.irq_counter |= 1;
            }
        } else if self.irq_counter == 0 {
            self.irq_counter = self.irq_counter_reload_value;
        } else {
            self.irq_counter -= 1;
        }

        if self.irq_enabled && self.irq_counter == 0 {
            self.irq_pending_delay_cycles = irq_pending_delay_cycles;
        }
    }
}

#[derive(PartialEq, Eq)]
enum IrqCounterReloadMode {
    Scanline,
    CpuCycle,
}
