use crate::memory::mapper::*;

const INITIAL_LAYOUT: InitialLayout = InitialLayout::builder()
    .prg_max_bank_count(32)
    .prg_bank_size(8 * KIBIBYTE)
    .prg_windows_by_board(&[(Board::Any, PRG_WINDOWS_C000_FIXED)])
    .chr_max_bank_count(256)
    .chr_bank_size(1 * KIBIBYTE)
    .chr_windows(CHR_BIG_WINDOWS_FIRST)
    .name_table_mirroring_source(NameTableMirroringSource::Cartridge)
    .build();

const PRG_WINDOWS_C000_FIXED: &[PrgWindow] = &[
    PrgWindow::new(0x6000, 0x6FFF, 4 * KIBIBYTE, PrgType::WorkRam),
    PrgWindow::new(0x7000, 0x71FF, KIBIBYTE / 2, PrgType::WorkRam),
    PrgWindow::new(0x7200, 0x73FF, KIBIBYTE / 2, PrgType::WorkRam),
    PrgWindow::new(0x7400, 0x7FFF, 3 * KIBIBYTE, PrgType::WorkRam),
    PrgWindow::new(0x8000, 0x9FFF, 8 * KIBIBYTE, PrgType::Banked(Rom, BankIndex::Register(P0))),
    PrgWindow::new(0xA000, 0xBFFF, 8 * KIBIBYTE, PrgType::Banked(Rom, BankIndex::Register(P1))),
    PrgWindow::new(0xC000, 0xDFFF, 8 * KIBIBYTE, PrgType::Banked(Rom, BankIndex::SECOND_LAST)),
    PrgWindow::new(0xE000, 0xFFFF, 8 * KIBIBYTE, PrgType::Banked(Rom, BankIndex::LAST)),
];

// Same as PRG_WINDOWS_C000_FIXED, except the 0x8000 and 0xC000 windows are swapped.
const PRG_WINDOWS_8000_FIXED: &[PrgWindow] = &[
    PrgWindow::new(0x6000, 0x6FFF, 4 * KIBIBYTE, PrgType::WorkRam),
    PrgWindow::new(0x7000, 0x71FF, KIBIBYTE / 2, PrgType::WorkRam),
    PrgWindow::new(0x7200, 0x73FF, KIBIBYTE / 2, PrgType::WorkRam),
    PrgWindow::new(0x7400, 0x7FFF, 3 * KIBIBYTE, PrgType::WorkRam),
    PrgWindow::new(0x8000, 0x9FFF, 8 * KIBIBYTE, PrgType::Banked(Rom, BankIndex::SECOND_LAST)),
    PrgWindow::new(0xA000, 0xBFFF, 8 * KIBIBYTE, PrgType::Banked(Rom, BankIndex::Register(P1))),
    PrgWindow::new(0xC000, 0xDFFF, 8 * KIBIBYTE, PrgType::Banked(Rom, BankIndex::Register(P0))),
    PrgWindow::new(0xE000, 0xFFFF, 8 * KIBIBYTE, PrgType::Banked(Rom, BankIndex::LAST)),
];

const CHR_BIG_WINDOWS_FIRST: &[ChrWindow] = &[
    // Big windows.
    ChrWindow::new(0x0000, 0x07FF, 2 * KIBIBYTE, ChrType(Rom, BankIndex::Register(C0))),
    ChrWindow::new(0x0800, 0x0FFF, 2 * KIBIBYTE, ChrType(Rom, BankIndex::Register(C1))),
    // Small windows.
    ChrWindow::new(0x1000, 0x13FF, 1 * KIBIBYTE, ChrType(Rom, BankIndex::Register(C2))),
    ChrWindow::new(0x1400, 0x17FF, 1 * KIBIBYTE, ChrType(Rom, BankIndex::Register(C3))),
    ChrWindow::new(0x1800, 0x1BFF, 1 * KIBIBYTE, ChrType(Rom, BankIndex::Register(C4))),
    ChrWindow::new(0x1C00, 0x1FFF, 1 * KIBIBYTE, ChrType(Rom, BankIndex::Register(C5))),
];

const CHR_SMALL_WINDOWS_FIRST: &[ChrWindow] = &[
    // Small windows.
    ChrWindow::new(0x0000, 0x03FF, 1 * KIBIBYTE, ChrType(Rom, BankIndex::Register(C2))),
    ChrWindow::new(0x0400, 0x07FF, 1 * KIBIBYTE, ChrType(Rom, BankIndex::Register(C3))),
    ChrWindow::new(0x0800, 0x0BFF, 1 * KIBIBYTE, ChrType(Rom, BankIndex::Register(C4))),
    ChrWindow::new(0x0C00, 0x0FFF, 1 * KIBIBYTE, ChrType(Rom, BankIndex::Register(C5))),
    // Big windows.
    ChrWindow::new(0x1000, 0x17FF, 2 * KIBIBYTE, ChrType(Rom, BankIndex::Register(C0))),
    ChrWindow::new(0x1800, 0x1FFF, 2 * KIBIBYTE, ChrType(Rom, BankIndex::Register(C1))),
];

// MMC3 (TSROM and others) and MMC6 (HKROM)
pub struct Mapper004 {
    selected_register_id: BankIndexRegisterId,

    irq_pending: bool,
    irq_enabled: bool,
    irq_counter: u8,
    force_reload_irq_counter: bool,
    irq_counter_reload_value: u8,
    irq_counter_suppression_cycles: u8,
    pattern_table_side: PatternTableSide,

    params: MapperParams,
}

impl Mapper for Mapper004 {
    fn write_to_cartridge_space(&mut self, address: CpuAddress, value: u8) {
        let is_even_address = address.to_raw() % 2 == 0;
        match address.to_raw() {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x5FFF => { /* Do nothing. */ }
            0x6000..=0x7FFF =>                    self.prg_memory_mut().write(address, value),
            0x8000..=0x9FFF if is_even_address => self.bank_select(value),
            0x8000..=0x9FFF =>                    self.set_bank_index(value),
            0xA000..=0xBFFF if is_even_address => self.set_mirroring(value),
            0xA000..=0xBFFF =>                    self.prg_ram_protect(value),
            0xC000..=0xDFFF if is_even_address => self.set_irq_reload_value(value),
            0xC000..=0xDFFF =>                    self.reload_irq_counter(),
            0xE000..=0xFFFF if is_even_address => self.disable_irq(),
            0xE000..=0xFFFF =>                    self.enable_irq(),
        }
    }

    fn on_end_of_ppu_cycle(&mut self) {
        if self.irq_counter_suppression_cycles > 0 {
            self.irq_counter_suppression_cycles -= 1;
        }
    }

    fn process_current_ppu_address(&mut self, address: PpuAddress) {
        if address.to_scroll_u16() >= 0x2000 {
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
            if self.irq_counter == 0 || self.force_reload_irq_counter {
                self.irq_counter = self.irq_counter_reload_value;
                self.force_reload_irq_counter = false;
            } else {
                self.irq_counter -= 1;
            }

            if self.irq_enabled && self.irq_counter == 0 {
                self.irq_pending = true;
            }
        }

        self.pattern_table_side = next_side;
    }

    fn irq_pending(&self) -> bool {
        self.irq_pending
    }

    fn params(&self) -> &MapperParams { &self.params }
    fn params_mut(&mut self) -> &mut MapperParams { &mut self.params }
}

impl Mapper004 {
    pub fn new(cartridge: &Cartridge) -> Result<Mapper004, String> {
        Ok(Mapper004 {
            selected_register_id: C0,

            irq_pending: false,
            irq_enabled: false,
            irq_counter: 0,
            force_reload_irq_counter: false,
            irq_counter_reload_value: 0,
            irq_counter_suppression_cycles: 0,
            pattern_table_side: PatternTableSide::Left,

            params: INITIAL_LAYOUT.make_mapper_params(cartridge, Board::Any),
        })
    }

    fn bank_select(&mut self, value: u8) {
        let chr_big_windows_first =                                 (value & 0b1000_0000) == 0;
        let prg_fixed_c000 =                                        (value & 0b0100_0000) == 0;
        //self.prg_ram_enabled =                                    (value & 0b0010_0000) != 0;
        self.selected_register_id = Mapper004::register_id_from_byte(value & 0b0000_0111);

        if chr_big_windows_first {
            self.chr_memory_mut().set_windows(CHR_BIG_WINDOWS_FIRST.clone())
        } else {
            self.chr_memory_mut().set_windows(CHR_SMALL_WINDOWS_FIRST.clone())
        }

        if prg_fixed_c000 {
            self.prg_memory_mut().set_windows(PRG_WINDOWS_C000_FIXED.clone());
        } else {
            self.prg_memory_mut().set_windows(PRG_WINDOWS_8000_FIXED.clone());
        }
    }

    fn set_bank_index(&mut self, value: u8) {
        let selected_register_id = self.selected_register_id;
        match selected_register_id {
            // Double-width windows can only use even banks.
            C0 | C1 => {
                let bank_index = u16::from(value & 0b1111_1110);
                self.chr_memory_mut().set_bank_index_register(selected_register_id, bank_index);
            }
            C2 | C3 | C4 | C5 => {
                let bank_index = u16::from(value);
                self.chr_memory_mut().set_bank_index_register(selected_register_id, bank_index);
            }
            // There can only be up to 64 PRG banks, though some ROM hacks use more.
            P0 | P1 => {
                assert_eq!(value & 0b1100_0000, 0, "ROM hack.");
                let bank_index = u16::from(value & 0b0011_1111);
                self.prg_memory_mut().set_bank_index_register(selected_register_id, bank_index);
            }
            _ => unreachable!("Bank Index Register ID {selected_register_id:?} is not used by mapper 4."),
        };
    }

    fn set_mirroring(&mut self, value: u8) {
        use NameTableMirroring::*;
        match (self.name_table_mirroring(), value & 0b0000_0001) {
            (Vertical, 1) => self.set_name_table_mirroring(Horizontal),
            (Horizontal, 0) => self.set_name_table_mirroring(Vertical),
            _ => { /* Other mirrorings cannot be changed. */ },
        }
    }

    fn prg_ram_protect(&mut self, _value: u8) {
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

    fn set_irq_reload_value(&mut self, value: u8) {
        self.irq_counter_reload_value = value;
    }

    fn reload_irq_counter(&mut self) {
        // TODO: This line probably isn't useful despite what the wiki says.
        self.irq_counter = 0;
        self.force_reload_irq_counter = true;
    }

    fn disable_irq(&mut self) {
        self.irq_enabled = false;
        self.irq_pending = false;
    }

    fn enable_irq(&mut self) {
        self.irq_enabled = true;
    }

    fn register_id_from_byte(value: u8) -> BankIndexRegisterId {
        use BankIndexRegisterId::*;
        match value {
            0b000 => C0,
            0b001 => C1,
            0b010 => C2,
            0b011 => C3,
            0b100 => C4,
            0b101 => C5,
            0b110 => P0,
            0b111 => P1,
            _ => unreachable!(),
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
}