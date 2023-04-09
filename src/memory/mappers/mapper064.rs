use crate::memory::mapper::*;

const PRG_WINDOWS_PRIMARY: PrgWindows = PrgWindows::new(&[
    PrgWindow::new(0x6000, 0x7FFF, 8 * KIBIBYTE, PrgType::Empty),
    PrgWindow::new(0x8000, 0x9FFF, 8 * KIBIBYTE, PrgType::VariableBank(Rom, P0)),
    PrgWindow::new(0xA000, 0xBFFF, 8 * KIBIBYTE, PrgType::VariableBank(Rom, P1)),
    PrgWindow::new(0xC000, 0xDFFF, 8 * KIBIBYTE, PrgType::VariableBank(Rom, P2)),
    PrgWindow::new(0xE000, 0xFFFF, 8 * KIBIBYTE, PrgType::ConstantBank(Rom, BankIndex::LAST)),
]);

const PRG_WINDOWS_SECONDARY: PrgWindows = PrgWindows::new(&[
    PrgWindow::new(0x6000, 0x7FFF, 8 * KIBIBYTE, PrgType::Empty),
    PrgWindow::new(0x8000, 0x9FFF, 8 * KIBIBYTE, PrgType::VariableBank(Rom, P2)),
    PrgWindow::new(0xA000, 0xBFFF, 8 * KIBIBYTE, PrgType::VariableBank(Rom, P1)),
    PrgWindow::new(0xC000, 0xDFFF, 8 * KIBIBYTE, PrgType::VariableBank(Rom, P0)),
    PrgWindow::new(0xE000, 0xFFFF, 8 * KIBIBYTE, PrgType::ConstantBank(Rom, BankIndex::LAST)),
]);

const CHR_BIG_WINDOWS_PRIMARY: ChrWindows = ChrWindows::new(&[
    // Big windows.
    ChrWindow::new(0x0000, 0x07FF, 2 * KIBIBYTE, ChrType::VariableBank(Rom, C0)),
    ChrWindow::new(0x0800, 0x0FFF, 2 * KIBIBYTE, ChrType::VariableBank(Rom, C1)),
    // Small windows.
    ChrWindow::new(0x1000, 0x13FF, 1 * KIBIBYTE, ChrType::VariableBank(Rom, C2)),
    ChrWindow::new(0x1400, 0x17FF, 1 * KIBIBYTE, ChrType::VariableBank(Rom, C3)),
    ChrWindow::new(0x1800, 0x1BFF, 1 * KIBIBYTE, ChrType::VariableBank(Rom, C4)),
    ChrWindow::new(0x1C00, 0x1FFF, 1 * KIBIBYTE, ChrType::VariableBank(Rom, C5)),
]);

const CHR_BIG_WINDOWS_SECONDARY: ChrWindows = ChrWindows::new(&[
    // Small windows.
    ChrWindow::new(0x0000, 0x03FF, 1 * KIBIBYTE, ChrType::VariableBank(Rom, C2)),
    ChrWindow::new(0x0400, 0x07FF, 1 * KIBIBYTE, ChrType::VariableBank(Rom, C3)),
    ChrWindow::new(0x0800, 0x0BFF, 1 * KIBIBYTE, ChrType::VariableBank(Rom, C4)),
    ChrWindow::new(0x0C00, 0x0FFF, 1 * KIBIBYTE, ChrType::VariableBank(Rom, C5)),
    // Big windows.
    ChrWindow::new(0x1000, 0x17FF, 2 * KIBIBYTE, ChrType::VariableBank(Rom, C0)),
    ChrWindow::new(0x1800, 0x1FFF, 2 * KIBIBYTE, ChrType::VariableBank(Rom, C1)),
]);

const CHR_SMALL_WINDOWS_PRIMARY: ChrWindows = ChrWindows::new(&[
    ChrWindow::new(0x0000, 0x03FF, 1 * KIBIBYTE, ChrType::VariableBank(Rom, C0)),
    ChrWindow::new(0x0400, 0x07FF, 1 * KIBIBYTE, ChrType::VariableBank(Rom, C6)),
    ChrWindow::new(0x0800, 0x0BFF, 1 * KIBIBYTE, ChrType::VariableBank(Rom, C1)),
    ChrWindow::new(0x0C00, 0x0FFF, 1 * KIBIBYTE, ChrType::VariableBank(Rom, C7)),
    ChrWindow::new(0x1000, 0x13FF, 1 * KIBIBYTE, ChrType::VariableBank(Rom, C2)),
    ChrWindow::new(0x1400, 0x17FF, 1 * KIBIBYTE, ChrType::VariableBank(Rom, C3)),
    ChrWindow::new(0x1800, 0x1BFF, 1 * KIBIBYTE, ChrType::VariableBank(Rom, C4)),
    ChrWindow::new(0x1C00, 0x1FFF, 1 * KIBIBYTE, ChrType::VariableBank(Rom, C5)),
]);

const CHR_SMALL_WINDOWS_SECONDARY: ChrWindows = ChrWindows::new(&[
    ChrWindow::new(0x0000, 0x03FF, 1 * KIBIBYTE, ChrType::VariableBank(Rom, C2)),
    ChrWindow::new(0x0400, 0x07FF, 1 * KIBIBYTE, ChrType::VariableBank(Rom, C3)),
    ChrWindow::new(0x0800, 0x0BFF, 1 * KIBIBYTE, ChrType::VariableBank(Rom, C4)),
    ChrWindow::new(0x0C00, 0x0FFF, 1 * KIBIBYTE, ChrType::VariableBank(Rom, C5)),
    ChrWindow::new(0x1000, 0x13FF, 1 * KIBIBYTE, ChrType::VariableBank(Rom, C0)),
    ChrWindow::new(0x1400, 0x17FF, 1 * KIBIBYTE, ChrType::VariableBank(Rom, C6)),
    ChrWindow::new(0x1800, 0x1BFF, 1 * KIBIBYTE, ChrType::VariableBank(Rom, C1)),
    ChrWindow::new(0x1C00, 0x1FFF, 1 * KIBIBYTE, ChrType::VariableBank(Rom, C7)),
]);

const CPU_CYCLE_MODE_IRQ_PENDING_DELAY: u8 = 1;
const SCANLINE_MODE_IRQ_PENDING_DELAY: u8 = 4;

// RAMBO-1 (Similar to MMC3)
pub struct Mapper064 {
    selected_register_id: BankIndexRegisterId,

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
            .prg_windows(PRG_WINDOWS_PRIMARY)
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
            0xA000..=0xBFFF if is_even_address => self.set_name_table_mirroring(params, value),
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

        if self.irq_counter_reload_mode == IrqCounterReloadMode::CpuCycle {
            if cycle % 4 == 0 {
                self.tick_irq_counter(CPU_CYCLE_MODE_IRQ_PENDING_DELAY);
            }
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
        let prg_windows = if value & 0b0100_0000 == 0 {
            PRG_WINDOWS_PRIMARY
        } else {
            PRG_WINDOWS_SECONDARY
        };
        params.prg_memory_mut().set_windows(prg_windows);

        let chr_windows = match value & 0b1010_0000 {
            0b0000_0000 => CHR_BIG_WINDOWS_PRIMARY,
            0b0010_0000 => CHR_SMALL_WINDOWS_PRIMARY,
            0b1000_0000 => CHR_BIG_WINDOWS_SECONDARY,
            0b1010_0000 => CHR_SMALL_WINDOWS_SECONDARY,
            _ => unreachable!(),
        };
        params.chr_memory_mut().set_windows(chr_windows);

        self.selected_register_id = match value & 0b0000_1111 {
            0b0000 => C0,
            0b0001 => C1,
            0b0010 => C2,
            0b0011 => C3,
            0b0100 => C4,
            0b0101 => C5,
            0b0110 => P0,
            0b0111 => P1,
            0b1000 => C6,
            0b1001 => C7,
            0b1011 => return,
            0b1101 => return,
            0b1111 => P2,
            _ => unreachable!(),
        };
    }

    fn set_bank_index(&self, params: &mut MapperParams, value: u8) {
        match self.selected_register_id {
            P0 | P1 | P2 => 
                params.prg_memory_mut().set_bank_index_register(self.selected_register_id, value),
            C0 | C1 | C2 | C3 | C4 | C5 | C6 | C7 =>
                params.chr_memory_mut().set_bank_index_register(self.selected_register_id, value),
            _ => unreachable!(),
        }
    }

    fn set_name_table_mirroring(&self, params: &mut MapperParams, value: u8) {
        let mirroring = if value & 1 == 0 {
            NameTableMirroring::Vertical
        } else {
            NameTableMirroring::Horizontal
        };
        params.set_name_table_mirroring(mirroring);
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
